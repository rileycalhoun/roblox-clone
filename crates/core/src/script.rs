//! Tiny Lua-subset scripting in Rust (std only).
//!
//! Attached to `Part.script`, one rule per line:
//! ```text
//! onJoin: say Welcome
//! onTouch: respawn
//! onTouch: say Ouch
//! tick: move 0,0,6 amplitude 6 freq 0.25
//! ```
//! Supported triggers: `onJoin`, `onTouch`, `tick`.
//! Supported actions: `say <text>`, `respawn`, `move <x,y,z> amplitude <a> freq <f>`.

use crate::math::Vec3;

#[derive(Debug, Clone, PartialEq)]
pub enum Trigger {
    Join,
    Touch,
    Tick,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Say(String),
    Respawn,
    Move { offset: Vec3, amplitude: f32, freq: f32 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    pub trigger: Trigger,
    pub action: Action,
}

pub fn parse_script(src: &str) -> Result<Vec<Rule>, String> {
    let mut rules = Vec::new();
    for (lineno, raw) in src.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (head, tail) = line
            .split_once(':')
            .ok_or_else(|| format!("line {}: missing ':'", lineno + 1))?;
        let trigger = match head.trim() {
            "onJoin" => Trigger::Join,
            "onTouch" => Trigger::Touch,
            "tick" => Trigger::Tick,
            other => return Err(format!("line {}: bad trigger '{other}'", lineno + 1)),
        };
        let body = tail.trim();
        let action = if let Some(text) = body.strip_prefix("say") {
            Action::Say(text.trim().trim_matches('"').to_string())
        } else if body == "respawn" {
            Action::Respawn
        } else if let Some(rest) = body.strip_prefix("move") {
            // move 0,0,6 amplitude 6 freq 0.25
            let toks: Vec<&str> = rest.trim().split_whitespace().collect();
            if toks.len() != 5 || toks[1] != "amplitude" || toks[3] != "freq" {
                return Err(format!("line {}: bad move syntax", lineno + 1));
            }
            let c: Vec<&str> = toks[0].split(',').collect();
            if c.len() != 3 {
                return Err(format!("line {}: bad move vec", lineno + 1));
            }
            let offset = Vec3::new(
                c[0].parse().map_err(|_| format!("line {}: bad x", lineno + 1))?,
                c[1].parse().map_err(|_| format!("line {}: bad y", lineno + 1))?,
                c[2].parse().map_err(|_| format!("line {}: bad z", lineno + 1))?,
            );
            let amplitude: f32 =
                toks[2].parse().map_err(|_| format!("line {}: bad amplitude", lineno + 1))?;
            let freq: f32 =
                toks[4].parse().map_err(|_| format!("line {}: bad freq", lineno + 1))?;
            Action::Move { offset, amplitude, freq }
        } else {
            return Err(format!("line {}: bad action", lineno + 1));
        };
        rules.push(Rule { trigger, action });
    }
    Ok(rules)
}

/// Mover displacement at time t (seconds): offset * sin(2*pi*freq*t) * amplitude scale.
/// Offset is treated as direction; amplitude scales it.
pub fn mover_offset(action: &Action, t: f32) -> Option<Vec3> {
    match action {
        Action::Move { offset, amplitude, freq } => {
            let s = (std::f32::consts::TAU * freq * t).sin() * amplitude;
            // offset encodes direction+distance; normalize-ish by using as-is scaled.
            // Convention: offset like 0,0,6 means full travel 6; amplitude multiplies 0..1 shape.
            Some(offset.scale(s / offset_length(*offset).max(1e-6)))
        }
        _ => None,
    }
}

fn offset_length(v: Vec3) -> f32 {
    (v.x * v.x + v.y * v.y + v.z * v.z).sqrt()
}

/// True if any rule on this script says touch->respawn.
pub fn is_kill_script(src: &str) -> bool {
    parse_script(src)
        .map(|rules| {
            rules.iter().any(|r| r.trigger == Trigger::Touch && r.action == Action::Respawn)
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_all() {
        let src = "onJoin: say Welcome\nonTouch: respawn\ntick: move 0,0,6 amplitude 6 freq 0.25\n";
        let rules = parse_script(src).unwrap();
        assert_eq!(rules.len(), 3);
    }

    #[test]
    fn mover_zero_at_zero() {
        let src = "tick: move 0,0,6 amplitude 6 freq 0.25";
        let rules = parse_script(src).unwrap();
        let off = mover_offset(&rules[0].action, 0.0).unwrap();
        assert!(off.z.abs() < 1e-5);
    }

    #[test]
    fn kill_detect() {
        assert!(is_kill_script("onTouch: respawn"));
        assert!(!is_kill_script("onJoin: say Hi"));
    }
}
