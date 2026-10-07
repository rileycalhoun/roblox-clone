//! Tiny line-based TCP protocol (std only).
//!
//! Client -> Server:
//!   `hello <name>\n`
//!   `input <seq> <tick> <fwd> <side> <jump:0|1>\n`
//! Server -> Client:
//!   `welcome <id>\n`
//!   `snapshot <tick> <id>:<x>,<y>,<z>;...\n`

use crate::math::Vec3;

#[derive(Debug, Clone, PartialEq)]
pub enum ClientMsg {
    Hello { name: String },
    Input { seq: u32, tick: u64, fwd: f32, side: f32, jump: bool },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServerMsg {
    Welcome { id: u32 },
    Snapshot { tick: u64, players: Vec<(u32, Vec3)> },
}

pub fn encode_client(m: &ClientMsg) -> String {
    match m {
        ClientMsg::Hello { name } => format!("hello {}\n", sanitize(name)),
        ClientMsg::Input { seq, tick, fwd, side, jump } => {
            format!("input {} {} {} {} {}\n", seq, tick, fwd, side, if *jump { 1 } else { 0 })
        }
    }
}

pub fn encode_server(m: &ServerMsg) -> String {
    match m {
        ServerMsg::Welcome { id } => format!("welcome {}\n", id),
        ServerMsg::Snapshot { tick, players } => {
            let mut s = format!("snapshot {} ", tick);
            for (i, (id, p)) in players.iter().enumerate() {
                if i > 0 {
                    s.push(';');
                }
                s.push_str(&format!("{}:{},{},{}", id, p.x, p.y, p.z));
            }
            s.push('\n');
            s
        }
    }
}

pub fn decode_client(line: &str) -> Option<ClientMsg> {
    let t = line.trim();
    let mut parts = t.split_whitespace();
    match parts.next()? {
        "hello" => {
            let name: Vec<&str> = parts.collect();
            Some(ClientMsg::Hello { name: name.join(" ") })
        }
        "input" => {
            let seq: u32 = parts.next()?.parse().ok()?;
            let tick: u64 = parts.next()?.parse().ok()?;
            let fwd: f32 = parts.next()?.parse().ok()?;
            let side: f32 = parts.next()?.parse().ok()?;
            let jump: u8 = parts.next()?.parse().ok()?;
            Some(ClientMsg::Input { seq, tick, fwd, side, jump: jump != 0 })
        }
        _ => None,
    }
}

pub fn decode_server(line: &str) -> Option<ServerMsg> {
    let t = line.trim();
    let mut parts = t.splitn(2, ' ');
    match parts.next()? {
        "welcome" => {
            let id: u32 = parts.next()?.trim().parse().ok()?;
            Some(ServerMsg::Welcome { id })
        }
        "snapshot" => {
            let rest = parts.next()?;
            let mut sp = rest.splitn(2, ' ');
            let tick: u64 = sp.next()?.parse().ok()?;
            let list = sp.next().unwrap_or("").trim();
            let mut players = Vec::new();
            if !list.is_empty() {
                for item in list.split(';') {
                    let mut kv = item.splitn(2, ':');
                    let id: u32 = kv.next()?.parse().ok()?;
                    let xyz = kv.next()?;
                    let c: Vec<&str> = xyz.split(',').collect();
                    if c.len() != 3 {
                        return None;
                    }
                    players.push((
                        id,
                        Vec3::new(c[0].parse().ok()?, c[1].parse().ok()?, c[2].parse().ok()?),
                    ));
                }
            }
            Some(ServerMsg::Snapshot { tick, players })
        }
        _ => None,
    }
}

fn sanitize(s: &str) -> String {
    s.chars().filter(|c| !c.is_control()).take(24).collect::<String>().replace(' ', "_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_client() {
        let m = ClientMsg::Input { seq: 7, tick: 99, fwd: 1.0, side: -0.5, jump: true };
        let s = encode_client(&m);
        assert_eq!(decode_client(&s), Some(m));
    }

    #[test]
    fn roundtrip_server() {
        let m = ServerMsg::Snapshot {
            tick: 12,
            players: vec![(1, Vec3::new(0.0, 2.5, 0.0)), (2, Vec3::new(5.0, 3.0, 1.0))],
        };
        let s = encode_server(&m);
        assert_eq!(decode_server(&s), Some(m));
    }
}
