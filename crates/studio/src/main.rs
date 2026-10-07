//! Studio CLI (Studio equivalent, v0.1 headless). Std only.
//!
//! Commands:
//!   new                              -> print default place to stdout
//!   list <place>                     -> list parts
//!   add <place> <name> <x,y,z> <sx,sy,sz> <r,g,b> [anchored]  -> append part (prints new file)
//!   move <place> <id> <x,y,z>        -> move part
//!   delete <place> <id>              -> remove part
//!   save-default <out>               -> write baseplate place
//!   preview <place> <out.ppm>        -> side-view PPM render (no deps)
//!   playtest [place] [ticks]         -> run local physics headlessly, print final pos
//!   assets                           -> list asset catalog keys
//!   asset-add <place> <key> <x,y,z>  -> instantiate catalog asset (prints new file)
//!   script-check <place> <id>        -> parse + print script rules
//!   demo-place <out>                 -> write playable demo (spawn+kill+mover+tower)

use clone_core::{Vec3, World};

fn parse_v3(s: &str) -> Vec3 {
    let p: Vec<&str> = s.split(',').collect();
    assert_eq!(p.len(), 3, "expected x,y,z");
    Vec3::new(p[0].parse().expect("x"), p[1].parse().expect("y"), p[2].parse().expect("z"))
}

fn parse_rgb(s: &str) -> (u8, u8, u8) {
    let p: Vec<&str> = s.split(',').collect();
    assert_eq!(p.len(), 3, "expected r,g,b");
    (p[0].parse().expect("r"), p[1].parse().expect("g"), p[2].parse().expect("b"))
}

fn load_place(path: &str) -> World {
    let text = std::fs::read_to_string(path).expect("read place");
    clone_core::load_str(&text).expect("parse place")
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: clone_studio <cmd> [args...]");
        std::process::exit(2);
    }
    match args[1].as_str() {
        "new" => {
            print!("{}", clone_core::save_str(&World::baseplate()));
        }
        "list" => {
            let w = load_place(&args[2]);
            for p in &w.parts {
                println!(
                    "{} '{}' pos={},{},{} size={},{},{} color={},{},{} anchored={}",
                    p.id,
                    p.name,
                    p.pos.x,
                    p.pos.y,
                    p.pos.z,
                    p.size.x,
                    p.size.y,
                    p.size.z,
                    p.color.0,
                    p.color.1,
                    p.color.2,
                    p.anchored
                );
            }
        }
        "add" => {
            // add <place> <name> <x,y,z> <sx,sy,sz> <r,g,b> [anchored=true|false]
            let mut w = load_place(&args[2]);
            let anchored = args.get(7).map(|s| s == "true").unwrap_or(true);
            let id = w.add_part(
                &args[3],
                parse_v3(&args[4]),
                parse_v3(&args[5]),
                parse_rgb(&args[6]),
                anchored,
            );
            eprintln!("added id={id}");
            print!("{}", clone_core::save_str(&w));
        }
        "move" => {
            let mut w = load_place(&args[2]);
            let id: u32 = args[3].parse().expect("id");
            let pos = parse_v3(&args[4]);
            w.get_mut(id).expect("part id").pos = pos;
            print!("{}", clone_core::save_str(&w));
        }
        "delete" => {
            let mut w = load_place(&args[2]);
            let id: u32 = args[3].parse().expect("id");
            assert!(w.remove_part(id), "no such part");
            print!("{}", clone_core::save_str(&w));
        }
        "save-default" => {
            std::fs::write(&args[2], clone_core::save_str(&World::baseplate())).expect("write");
            eprintln!("wrote {}", &args[2]);
        }
        "preview" => {
            let w = load_place(&args[2]);
            let players = vec![(w.spawn.pos.x, w.spawn.pos.y, w.spawn.pos.z)];
            let ppm = clone_core::preview::render_side(&w, &players, 320, 200);
            std::fs::write(&args[3], ppm.to_bytes()).expect("write ppm");
            eprintln!("wrote {}", &args[3]);
        }
        "playtest" => {
            let w = if args.len() > 2 { load_place(&args[2]) } else { World::baseplate() };
            let ticks: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(300);
            let mut play = clone_player_local(w);
            for i in 0..ticks {
                let input = clone_core::Input {
                    fwd: 1.0,
                    side: 0.0,
                    jump: i == 120,
                };
                play.step(&input, 1.0 / 60.0);
            }
            let p = play.player_pos();
            println!("final pos={},{},{} on_ground={}", p.x, p.y, p.z, play.on_ground());
        }
        "assets" => {
            for a in clone_core::assets::CATALOG {
                println!("{} '{}' kind={} size={:?} color={:?}", a.key, a.label, a.kind.as_str(), a.size, a.color);
            }
        }
        "asset-add" => {
            let mut w = load_place(&args[2]);
            let pos = parse_v3(&args[4]);
            let id = clone_core::assets::instantiate(&mut w, &args[3], pos).expect("unknown asset key");
            eprintln!("added asset id={id}");
            print!("{}", clone_core::save_str(&w));
        }
        "script-check" => {
            let w = load_place(&args[2]);
            let id: u32 = args[3].parse().expect("id");
            let p = w.get(id).expect("part id");
            let rules = clone_core::script::parse_script(&p.script).expect("parse script");
            for r in &rules {
                println!("{:?} -> {:?}", r.trigger, r.action);
            }
            if rules.is_empty() {
                println!("(no rules)");
            }
        }
        "demo-place" => {
            let mut w = World::baseplate();
            w.spawn.pos = Vec3::new(0.0, 5.0, 0.0);
            clone_core::assets::instantiate(&mut w, "spawn", Vec3::new(-6.0, 1.0, 0.0));
            clone_core::assets::instantiate(&mut w, "kill", Vec3::new(6.0, 1.0, 0.0));
            clone_core::assets::instantiate(&mut w, "mover", Vec3::new(0.0, 3.0, -10.0));
            w.add_part("Tower", Vec3::new(0.0, 5.0, 10.0), Vec3::new(4.0, 10.0, 4.0), (200, 50, 50), true);
            std::fs::write(&args[2], clone_core::save_str(&w)).expect("write");
            eprintln!("wrote demo {}", &args[2]);
        }
        other => {
            eprintln!("unknown cmd: {other}");
            std::process::exit(2);
        }
    }
}

struct LocalWrap {
    world: World,
    player: clone_core::PlayerState,
}

fn clone_player_local(world: World) -> LocalWrap {
    let spawn = world.spawn.pos;
    LocalWrap { world, player: clone_core::PlayerState::at(spawn) }
}

impl LocalWrap {
    fn step(&mut self, input: &clone_core::Input, dt: f32) {
        clone_core::step_player(&self.world, &mut self.player, input, dt);
    }
    fn player_pos(&self) -> Vec3 {
        self.player.pos
    }
    fn on_ground(&self) -> bool {
        self.player.on_ground
    }
}
