//! Player binary: connects to server and walks forward headlessly.
//! Usage: clone_player <addr:port> <name>
//! MVP demo: sends forward input for ~5s, prints snapshots.

use clone_core::Input;
use clone_player::NetClient;
use std::time::{Duration, Instant};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: clone_player <addr:port> <name>");
        std::process::exit(2);
    }
    let mut client = NetClient::connect(&args[1], &args[2]).expect("connect");
    println!("connected as id={}", client.id);
    let start = Instant::now();
    let mut tick = 0u64;
    let input = Input { fwd: 1.0, side: 0.0, jump: false };
    while start.elapsed() < Duration::from_secs(5) {
        client.send_input(tick, &input).expect("send");
        match client.read_snapshot() {
            Ok(msg) => println!("{msg:?}"),
            Err(e) => {
                eprintln!("read: {e}");
                break;
            }
        }
        tick += 1;
    }
}
