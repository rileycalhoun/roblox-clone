//! Headless player client + local single-player sim. Std only.

use clone_core::net::{decode_server, encode_client, ClientMsg, ServerMsg};
use clone_core::{Input, PlayerState, Vec3, World};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
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
