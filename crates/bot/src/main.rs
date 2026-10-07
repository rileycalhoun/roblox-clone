//! Snapshot-driven TCP player. Usage: clone_bot <addr> <name> [seconds] [kill|patrol]

use clone_core::net::ServerMsg;
use clone_core::{Input, Vec3};
use clone_player::NetClient;
use std::io;
use std::time::{Duration, Instant};

fn distance(a: Vec3, b: Vec3) -> f32 {
    ((a.x - b.x).powi(2) + (a.z - b.z).powi(2)).sqrt()
}

fn run() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 || args.len() > 5 {
        eprintln!("usage: clone_bot <addr> <name> [seconds] [kill|patrol]");
        std::process::exit(2);
    }
    let seconds: u64 = args.get(3).map(|s| s.parse().map_err(|_| io::Error::other("invalid seconds"))).transpose()?.unwrap_or(10);
    let kill_route = args.get(4).is_some_and(|s| s == "kill");
    let waypoints: &[(f32, f32)] = if kill_route {
        &[(6.0, 0.0), (16.0, 0.0), (16.0, -14.0), (-14.0, -14.0), (-14.0, 12.0)]
    } else {
        &[(-16.0, 0.0), (-16.0, -14.0), (12.0, -14.0), (12.0, 14.0)]
    };
    let mut client = NetClient::connect(&args[1], &args[2])?;
    client.set_read_timeout(Duration::from_millis(250))?;
    let id = client.id;
    println!("{} welcome id={id}", args[2]);
    let started = Instant::now();
    let deadline = started + Duration::from_secs(seconds);
    let mut waypoint = 0;
    let mut last_pos: Option<Vec3> = None;
    let mut last_tick = 0;
    let mut path = 0.0f32;
    let mut snapshots = 0;
    let mut respawns = 0;
    let mut both = 0;
    let mut last_report = Instant::now();
    let mut checked_at = Instant::now();
    let mut checked_pos: Option<Vec3> = None;
    let mut jump_until = Instant::now();
    let mut last_jump = Instant::now();

    while Instant::now() < deadline {
        let (tick, players) = match client.read_snapshot() {
            Ok(ServerMsg::Snapshot { tick, players }) => (tick, players),
            Ok(_) => continue,
            Err(e) if matches!(e.kind(), io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted) => continue,
            Err(e) => return Err(e),
        };
        let Some(pos) = players.iter().find(|(player_id, _)| *player_id == id).map(|(_, p)| *p) else {
            continue;
        };
        let now = Instant::now();
        snapshots += 1;
        if players.len() >= 2 {
            both += 1;
            if both == 1 {
                println!("{} both_ids tick={tick} ids={:?}", args[2], players.iter().map(|(id, _)| id).collect::<Vec<_>>());
            }
        }
        if let Some(previous) = last_pos {
            let step = distance(previous, pos);
            // A jump back to spawn after entering the kill area is a server-observed respawn.
            if kill_route && previous.x > 3.0 && pos.x.abs() < 1.0 && pos.z.abs() < 1.0 && pos.y > 3.8 && step > 2.0 {
                respawns += 1;
                println!("{} respawn tick={tick} from=({:.2},{:.2},{:.2}) to=({:.2},{:.2},{:.2})", args[2], previous.x, previous.y, previous.z, pos.x, pos.y, pos.z);
                waypoint = 1;
            } else if step < 2.0 {
                path += step;
            }
        }
        last_pos = Some(pos);
        last_tick = tick;

        let target = waypoints[waypoint];
        if distance(pos, Vec3::new(target.0, pos.y, target.1)) < 1.5 {
            waypoint = (waypoint + 1) % waypoints.len();
            println!("{} waypoint={} tick={tick} pos=({:.2},{:.2},{:.2})", args[2], waypoint, pos.x, pos.y, pos.z);
        }
        let stuck = if now.duration_since(checked_at) >= Duration::from_millis(850) {
            let stuck = checked_pos.is_some_and(|p| distance(p, pos) < 0.5);
            checked_at = now;
            checked_pos = Some(pos);
            stuck
        } else {
            false
        };
        if stuck || now.duration_since(last_jump) >= Duration::from_millis(1300) {
            jump_until = now + Duration::from_millis(170);
            last_jump = now;
            println!("{} jump tick={tick} stuck={stuck} pos=({:.2},{:.2},{:.2})", args[2], pos.x, pos.y, pos.z);
        }
        let target = waypoints[waypoint];
        let input = Input {
            side: ((target.0 - pos.x) / 2.0).clamp(-1.0, 1.0),
            fwd: ((target.1 - pos.z) / 2.0).clamp(-1.0, 1.0),
            jump: now < jump_until,
        };
        client.send_input(tick, &input)?;
        if now.duration_since(last_report) >= Duration::from_secs(1) || snapshots == 1 {
            println!("{} progress tick={tick} pos=({:.2},{:.2},{:.2}) path={:.2} snapshots={snapshots} peers={}", args[2], pos.x, pos.y, pos.z, path, players.len());
            last_report = now;
        }
    }
    println!("{} summary id={id} tick={last_tick} pos={:?} path={path:.2} snapshots={snapshots} both_ids={both} respawns={respawns}", args[2], last_pos);
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("bot error: {e}");
        std::process::exit(1);
    }
}
