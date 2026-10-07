//! Headless player client + local single-player sim. Std only.

use clone_core::net::{decode_server, encode_client, ClientMsg, ServerMsg};
use clone_core::{Input, PlayerState, Vec3, World};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::mpsc;
use std::time::Duration;

/// Networked client. Blocking line protocol.
pub struct NetClient {
    reader: BufReader<TcpStream>,
    writer: TcpStream,
    pub id: u32,
    seq: u32,
}

impl NetClient {
    pub fn connect(addr: &str, name: &str) -> std::io::Result<Self> {
        let stream = TcpStream::connect(addr)?;
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        let writer = stream.try_clone()?;
        let reader = BufReader::new(stream);
        let mut c = Self { reader: reader_try_clone(&reader), writer, id: 0, seq: 0 };
        // hello
        let hello = encode_client(&ClientMsg::Hello { name: name.to_string() });
        c.writer.write_all(hello.as_bytes())?;
        let mut line = String::new();
        c.reader.read_line(&mut line).map_err(|e| {
            std::io::Error::new(e.kind(), format!("welcome read failed: {e}"))
        })?;
        match decode_server(&line) {
            Some(ServerMsg::Welcome { id }) => c.id = id,
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("bad welcome: {line}"),
                ))
            }
        }
        c.reader = reader;
        Ok(c)
    }

    pub fn send_input(&mut self, tick: u64, input: &Input) -> std::io::Result<()> {
        self.seq += 1;
        let m = ClientMsg::Input {
            seq: self.seq,
            tick,
            fwd: input.fwd,
            side: input.side,
            jump: input.jump,
        };
        self.writer.write_all(encode_client(&m).as_bytes())
    }

    pub fn set_read_timeout(&self, timeout: Duration) -> std::io::Result<()> {
        self.reader.get_ref().set_read_timeout(Some(timeout))
    }

    pub fn read_snapshot(&mut self) -> std::io::Result<ServerMsg> {
        let mut line = String::new();
        self.reader.read_line(&mut line)?;
        decode_server(&line).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, format!("bad snapshot: {line}"))
        })
    }
}

fn reader_try_clone(reader: &BufReader<TcpStream>) -> BufReader<TcpStream> {
    let s = reader.get_ref().try_clone().expect("clone stream");
    BufReader::new(s)
}

/// Local single-player simulation for offline play / playtest.
pub struct LocalPlay {
    pub world: World,
    pub player: PlayerState,
}

impl LocalPlay {
    pub fn new(world: World) -> Self {
        let spawn = world.spawn.pos;
        Self { world, player: PlayerState::at(spawn) }
    }

    pub fn step(&mut self, input: &Input, dt: f32) {
        clone_core::step_player(&self.world, &mut self.player, input, dt);
    }

    pub fn player_pos(&self) -> Vec3 {
        self.player.pos
    }
}

/// Non-blocking pump client for real-time frames (GUI play-with-friends path).
/// Owns an IO thread: the frame loop `send`s inputs and `poll`s the latest
/// player positions without ever blocking on the socket.
pub struct PumpClient {
    pub id: u32,
    tx: mpsc::Sender<Input>,
    rx: mpsc::Receiver<Vec<(f32, f32, f32)>>,
}

impl PumpClient {
    pub fn connect(addr: &str, name: &str) -> std::io::Result<Self> {
        let stream = TcpStream::connect(addr)?;
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        let mut w = stream.try_clone()?;
        let mut r = BufReader::new(stream);
        let hello = encode_client(&ClientMsg::Hello { name: name.to_string() });
        w.write_all(hello.as_bytes())?;
        let mut line = String::new();
        r.read_line(&mut line)?;
        let id = match decode_server(&line) {
            Some(ServerMsg::Welcome { id }) => id,
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "bad welcome",
                ))
            }
        };
        let (itx, irx) = mpsc::channel::<Input>();
        let (otx, orx) = mpsc::channel::<Vec<(f32, f32, f32)>>();
        std::thread::spawn(move || {
            let mut writer = w;
            let mut reader = r;
            let mut seq = 0u32;
            let mut tick = 0u64;
            loop {
                let mut cur: Option<Input> = None;
                while let Ok(inp) = irx.try_recv() {
                    cur = Some(inp);
                }
                if let Some(inp) = cur {
                    seq += 1;
                    tick += 1;
                    let msg = ClientMsg::Input {
                        seq,
                        tick,
                        fwd: inp.fwd,
                        side: inp.side,
                        jump: inp.jump,
                    };
                    if writer.write_all(encode_client(&msg).as_bytes()).is_err() {
                        break;
                    }
                }
                reader
                    .get_ref()
                    .set_read_timeout(Some(Duration::from_millis(5)))
                    .ok();
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(0) => break,
                    Ok(_) => {
                        if let Some(ServerMsg::Snapshot { players, .. }) =
                            decode_server(&line)
                        {
                            let list: Vec<(f32, f32, f32)> =
                                players.iter().map(|(_, v)| (v.x, v.y, v.z)).collect();
                            let _ = otx.send(list);
                        }
                    }
                    Err(_) => {}
                }
                std::thread::sleep(Duration::from_millis(16));
            }
        });
        Ok(Self { id, tx: itx, rx: orx })
    }

    pub fn send(&mut self, input: Input) {
        let _ = self.tx.send(input);
    }

    pub fn poll(&mut self, out: &mut Vec<(f32, f32, f32)>) {
        while let Ok(list) = self.rx.try_recv() {
            *out = list;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    #[test]
    fn pump_client_handshake_and_snapshots() {
        // Scripted fake server on an ephemeral port.
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().unwrap().to_string();
        let server = std::thread::spawn(move || {
            let (stream, _) = listener.accept().expect("accept");
            stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut w = stream.try_clone().unwrap();
            let mut r = BufReader::new(stream);
            let mut line = String::new();
            r.read_line(&mut line).expect("hello");
            assert!(line.starts_with("hello"), "got {line}");
            w.write_all(b"welcome 7\n").expect("welcome");
            // Drain a couple of inputs, then push snapshots.
            for _ in 0..3 {
                let mut l = String::new();
                r.read_line(&mut l).ok();
            }
            for _ in 0..5 {
                w.write_all(b"snapshot 3 7:1,2,3\n").expect("snap");
                std::thread::sleep(Duration::from_millis(10));
            }
        });

        let mut client = PumpClient::connect(&addr, "tester").expect("connect");
        assert_eq!(client.id, 7);
        for _ in 0..3 {
            client.send(Input { fwd: 1.0, side: 0.0, jump: false });
            std::thread::sleep(Duration::from_millis(20));
        }
        let mut out = Vec::new();
        let start = std::time::Instant::now();
        while out != vec![(1.0f32, 2.0, 3.0)] && start.elapsed() < Duration::from_secs(3) {
            client.poll(&mut out);
            std::thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(out, vec![(1.0f32, 2.0, 3.0)]);
        server.join().expect("server thread");
    }
}
