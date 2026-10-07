//! Asset catalog: named buildables that map to Part kinds + script templates.
//! Std only. Lets Studio offer "blocks, spawn, kill-brick, moving platform".

use crate::model::PartKind;
use crate::Vec3;
use crate::World;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Asset {
    pub key: &'static str,
    pub label: &'static str,
    pub kind: PartKind,
    pub size: [f32; 3],
    pub color: (u8, u8, u8),
    pub anchored: bool,
    pub script_template: &'static str,
}

pub const CATALOG: &[Asset] = &[
    Asset {
        key: "block",
        label: "Block 4x1x4",
        kind: PartKind::Block,
        size: [4.0, 1.0, 4.0],
        color: (180, 180, 180),
        anchored: true,
        script_template: "",
    },
    Asset {
        key: "spawn",
        label: "Spawn Pad 4x1x4",
        kind: PartKind::Block,
        size: [4.0, 1.0, 4.0],
        color: (80, 200, 120),
        anchored: true,
        script_template: "onJoin: say Welcome\n",
    },
    Asset {
        key: "kill",
        label: "Kill Brick 4x1x4",
        kind: PartKind::KillBrick,
        size: [4.0, 1.0, 4.0],
        color: (220, 40, 40),
        anchored: true,
        script_template: "onTouch: respawn\n",
    },
    Asset {
        key: "mover",
        label: "Moving Platform 6x1x6",
        kind: PartKind::Mover,
        size: [6.0, 1.0, 6.0],
        color: (80, 140, 230),
        anchored: true,
        script_template: "tick: move 0,0,6 amplitude 6 freq 0.25\n",
    },
];

pub fn find_asset(key: &str) -> Option<Asset> {
    CATALOG.iter().find(|a| a.key == key).copied()
}

/// Instantiate an asset into the world at a position. Returns new part id.
pub fn instantiate(world: &mut World, key: &str, pos: Vec3) -> Option<u32> {
    let a = find_asset(key)?;
    Some(world.add_part_kind(
        a.label,
        pos,
        Vec3::new(a.size[0], a.size[1], a.size[2]),
        a.color,
        a.anchored,
        a.kind,
        a.script_template,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_four() {
        assert_eq!(CATALOG.len(), 4);
    }

    #[test]
    fn instantiate_kill_brick() {
        let mut w = World::empty();
        let id = instantiate(&mut w, "kill", Vec3::new(0.0, 2.0, 0.0)).unwrap();
        let p = w.get(id).unwrap();
        assert_eq!(p.kind, PartKind::KillBrick);
        assert!(p.script.contains("respawn"));
    }
}
