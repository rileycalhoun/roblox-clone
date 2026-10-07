//! `.rplace` text format, hand-rolled (no serde, std only).
//!
//! ```text
//! RPLACE1
//! spawn 0 5 0
//! part 1 Baseplate 0,-0.5,0 64,1,64 100,150,100 anchored=true
//! ```

use crate::math::Vec3;
use crate::model::World;

fn parse_vec3(s: &str) -> Option<Vec3> {
    let c: Vec<&str> = s.split(',').collect();
    if c.len() != 3 {
        return None;
    }
    Some(Vec3::new(c[0].parse().ok()?, c[1].parse().ok()?, c[2].parse().ok()?))
}

fn parse_rgb(s: &str) -> Option<(u8, u8, u8)> {
    let c: Vec<&str> = s.split(',').collect();
    if c.len() != 3 {
        return None;
    }
    Some((c[0].parse().ok()?, c[1].parse().ok()?, c[2].parse().ok()?))
}

fn fmt_vec3(v: Vec3) -> String {
    format!("{},{},{}", v.x, v.y, v.z)
}

pub fn save_str(world: &World) -> String {
    let mut out = String::from("RPLACE1\n");
    out.push_str(&format!(
        "spawn {} {} {}\n",
        world.spawn.pos.x, world.spawn.pos.y, world.spawn.pos.z
    ));
    for p in &world.parts {
        out.push_str(&format!(
            "part {} {} {} {} {},{},{} anchored={} kind={} script=\"{}\"\n",
            p.id,
            escape_name(&p.name),
            fmt_vec3(p.pos),
            fmt_vec3(p.size),
            p.color.0,
            p.color.1,
            p.color.2,
            p.anchored,
            p.kind.as_str(),
            escape_script(&p.script),
        ));
    }
    out
}

fn escape_script(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "'").replace('\n', "|")
}

fn unescape_script(s: &str) -> String {
    let t = s.trim();
    let t = t.strip_prefix('"').unwrap_or(t);
    let t = t.strip_suffix('"').unwrap_or(t);
    t.replace('|', "\n")
}

fn escape_name(n: &str) -> String {
    if n.contains(' ') {
        format!("\"{}\"", n.replace('"', "'"))
    } else {
        n.to_string()
    }
}

fn unescape_name(n: &str) -> String {
    let t = n.trim();
    if t.starts_with('"') && t.ends_with('"') && t.len() >= 2 {
        t[1..t.len() - 1].to_string()
    } else {
        t.to_string()
    }
}

pub fn load_str(s: &str) -> Result<World, String> {
    let mut lines = s.lines();
    let head = lines.next().ok_or("empty file")?.trim();
    if head != "RPLACE1" {
        return Err(format!("bad header: {head}"));
    }
    let mut world = World::empty();
    let mut max_id = 0u32;
    for (i, raw) in lines.enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let toks: Vec<String> = tokenize(line);
        if toks.is_empty() {
            continue;
        }
        match toks[0].as_str() {
            "spawn" => {
                if toks.len() != 4 {
                    return Err(format!("line {}: bad spawn", i + 2));
                }
                let x: f32 = toks[1].parse().map_err(|_| "bad spawn x")?;
                let y: f32 = toks[2].parse().map_err(|_| "bad spawn y")?;
                let z: f32 = toks[3].parse().map_err(|_| "bad spawn z")?;
                world.spawn.pos = Vec3::new(x, y, z);
            }
            "part" => {
                if toks.len() < 7 {
                    return Err(format!("line {}: bad part ({} toks)", i + 2, toks.len()));
                }
                let id: u32 = toks[1].parse().map_err(|_| "bad id")?;
                let name = unescape_name(&toks[2]);
                let pos = parse_vec3(&toks[3]).ok_or("bad pos")?;
                let size = parse_vec3(&toks[4]).ok_or("bad size")?;
                let color = parse_rgb(&toks[5]).ok_or("bad color")?;
                let anchored = match toks[6].as_str() {
                    "anchored=true" => true,
                    "anchored=false" => false,
                    _ => return Err("bad anchored".into()),
                };
                let mut kind = crate::model::PartKind::Block;
                let mut script = String::new();
                for extra in toks.iter().skip(7) {
                    if let Some(k) = extra.strip_prefix("kind=") {
                        kind = crate::model::PartKind::from_str(k).ok_or("bad kind")?;
                    } else if let Some(s) = extra.strip_prefix("script=") {
                        script = unescape_script(s);
                    } else {
                        return Err(format!("line {}: bad extra '{extra}'", i + 2));
                    }
                }
                max_id = max_id.max(id);
                world.parts.push(crate::model::Part { id, name, pos, size, color, anchored, kind, script });
            }
            other => return Err(format!("line {}: unknown '{other}'", i + 2)),
        }
    }
    world.next_id = max_id + 1;
    if world.next_id == 0 {
        world.next_id = 1;
    }
    Ok(world)
}

/// Split respecting double quotes.
fn tokenize(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_q = false;
    for ch in line.chars() {
        match ch {
            '"' => {
                in_q = !in_q;
                cur.push(ch);
            }
            ' ' | '\t' if !in_q => {
                if !cur.is_empty() {
                    out.push(cur.clone());
                    cur.clear();
                }
            }
            _ => cur.push(ch),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let mut w = World::baseplate();
        w.spawn.pos = Vec3::new(1.0, 5.0, 2.0);
        w.add_part("Tower", Vec3::new(0.0, 5.0, 10.0), Vec3::new(4.0, 10.0, 4.0), (200, 50, 50), true);
        let s = save_str(&w);
        let back = load_str(&s).expect("load");
        assert_eq!(back.parts.len(), w.parts.len());
        assert_eq!(back.spawn.pos, w.spawn.pos);
    }
}
