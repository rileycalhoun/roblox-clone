//! Dedicated server binary. Std only.
//! Usage: clone_server <bind_addr:port> [place.rplace]

use clone_core::net::{decode_client, encode_server, ClientMsg, ServerMsg};
use clone_core::{Input, World};
use clone_server::GameServer;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: clone_server <bind_addr:port> [place.rplace]");
        std::process::exit(2);
    }
    let bind = &args[1];
    let world = if args.len() >= 3 {
        let text = std::fs::read_to_string(&args[2]).expect("read place file");
        clone_core::load_str(&text).expect("parse place file")
    } else {
        World::baseplate()
    };

    let game = Arc::new(Mutex::new(GameServer::new(world)));
    let subs: Arc<Mutex<Vec<mpsc::Sender<String>>>> = Arc::new(Mutex::new(Vec::new()));

    // Simulation thread: 60 Hz physics, 20 Hz snapshots.
    {
        let game = game.clone();
        let subs = subs.clone();
        std::thread::spawn(move || {
            let mut last_snap = Instant::now();
            loop {
                {
                    let mut g = game.lock().unwrap();
                    g.tick_once(1.0 / 60.0);
                    if last_snap.elapsed() >= Duration::from_millis(50) {
                        last_snap = Instant::now();
                        let msg = ServerMsg::Snapshot { tick: g.tick, players: g.positions() };
                        let line = encode_server(&msg);
                        let mut subs = subs.lock().unwrap();
                        subs.retain(|tx| tx.send(line.clone()).is_ok());
                    }
                }
                std::thread::sleep(Duration::from_millis(16));
            }
        });
    }

    let listener = TcpListener::bind(bind).expect("bind");
    println!("serving on {bind}");
    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                let game = game.clone();
                let subs = subs.clone();
                std::thread::spawn(move || handle_client(s, game, subs));
            }
            Err(e) => eprintln!("accept error: {e}"),
        }
    }
}

fn handle_client(
    stream: TcpStream,
    game: Arc<Mutex<GameServer>>,
    subs: Arc<Mutex<Vec<mpsc::Sender<String>>>>,
) {
    let peer = stream.peer_addr().map(|a| a.to_string()).unwrap_or_default();
    let mut reader = BufReader::new(stream.try_clone().expect("clone"));
    let mut writer = stream;
    writer.set_write_timeout(Some(Duration::from_secs(5))).ok();

    // Expect hello first.
    let mut line = String::new();
    if reader.read_line(&mut line).unwrap_or(0) == 0 {
        return;
    }
    let name = match decode_client(&line) {
        Some(ClientMsg::Hello { name }) => name,
        _ => {
            let _ = writer.write_all(b"welcome 0\n");
            return;
        }
    };
    let _ = name;

    let id = game.lock().unwrap().add_player();
    let _ = writer.write_all(encode_server(&ServerMsg::Welcome { id }).as_bytes());

    let (tx, rx) = mpsc::channel::<String>();
    subs.lock().unwrap().push(tx);

    // Forward snapshots to this socket.
    let mut w2 = writer.try_clone().expect("clone2");
    std::thread::spawn(move || {
        for line in rx {
            if w2.write_all(line.as_bytes()).is_err() {
                break;
            }
        }
    });

    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(_) => break,
        }
        if let Some(ClientMsg::Input { fwd, side, jump, .. }) = decode_client(&line) {
            game.lock()
                .unwrap()
                .set_input(id, Input { fwd: fwd.clamp(-1.0, 1.0), side: side.clamp(-1.0, 1.0), jump });
        }
        let _ = peer.as_str();
    }
    game.lock().unwrap().remove_player(id);
}
