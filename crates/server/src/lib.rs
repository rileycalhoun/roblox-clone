use clone_core::{Input, PlayerState, Vec3, World};
use std::collections::HashMap;

/// Authoritative simulation. No sockets here so it stays testable.
pub struct GameServer {
    pub world: World,
    pub players: HashMap<u32, PlayerState>,
    pub inputs: HashMap<u32, Input>,
    pub tick: u64,
    pub time: f32,
    next_id: u32,
    mover_base: HashMap<u32, Vec3>,
}

impl GameServer {
    pub fn new(world: World) -> Self {
        let mut mover_base = HashMap::new();
        for p in &world.parts {
            if p.kind == clone_core::PartKind::Mover {
                mover_base.insert(p.id, p.pos);
            }
        }
        Self { world, players: HashMap::new(), inputs: HashMap::new(), tick: 0, time: 0.0, next_id: 1, mover_base }
    }

    pub fn add_player(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        let spawn = self.world.spawn.pos;
        self.players.insert(id, PlayerState::at(spawn));
        self.inputs.insert(id, Input::default());
        id
    }

    pub fn remove_player(&mut self, id: u32) {
        self.players.remove(&id);
        self.inputs.remove(&id);
    }

    pub fn set_input(&mut self, id: u32, input: Input) {
        if let Some(slot) = self.inputs.get_mut(&id) {
            *slot = input;
        }
    }

    pub fn tick_once(&mut self, dt: f32) {
        self.tick += 1;
        self.time += dt;
        // Moving platforms from tick:move scripts.
        for part in self.world.parts.iter_mut() {
            if part.kind != clone_core::PartKind::Mover {
                continue;
            }
            let base = self.mover_base.get(&part.id).copied().unwrap_or(part.pos);
            if let Ok(rules) = clone_core::script::parse_script(&part.script) {
                for r in rules {
                    if r.trigger == clone_core::script::Trigger::Tick {
                        if let Some(off) = clone_core::script::mover_offset(&r.action, self.time) {
                            part.pos = base.add(off);
                            break;
                        }
                    }
                }
            }
        }
        let ids: Vec<u32> = self.players.keys().copied().collect();
        for id in ids {
            let input = self.inputs.get(&id).copied().unwrap_or_default();
            if let Some(p) = self.players.get_mut(&id) {
                clone_core::step_player(&self.world, p, &input, dt);
            }
        }
        // Kill bricks: touch -> respawn.
        let spawn = self.world.spawn.pos;
        for p in self.players.values_mut() {
            let min = p.min();
            let max = p.max();
            let touched_kill = self.world.parts.iter().any(|part| {
                let kill_kind = part.kind == clone_core::PartKind::KillBrick;
                let kill_script = clone_core::script::is_kill_script(&part.script);
                (kill_kind || kill_script) && part.overlaps(min, max)
            });
            if touched_kill {
                p.pos = spawn;
                p.vel = Vec3::ZERO;
            }
        }
    }

    pub fn positions(&self) -> Vec<(u32, Vec3)> {
        self.players.iter().map(|(id, p)| (*id, p.pos)).collect()
    }

    /// Re-anchor moving platform bases after Studio edits.
    pub fn mover_reset(&mut self) {
        self.mover_base.clear();
        for p in &self.world.parts {
            if p.kind == clone_core::PartKind::Mover {
                self.mover_base.insert(p.id, p.pos);
            }
        }
    }

    /// Write mover base positions back into the world (so saving after a
    /// play session persists the authored layout, not a mid-oscillation pose).
    pub fn restore_mover_bases(&mut self) {
        for part in self.world.parts.iter_mut() {
            if part.kind == clone_core::PartKind::Mover {
                if let Some(base) = self.mover_base.get(&part.id) {
                    part.pos = *base;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_falls_to_ground() {
        let mut srv = GameServer::new(World::baseplate());
        let id = srv.add_player();
        for _ in 0..180 {
            srv.tick_once(1.0 / 60.0);
        }
        let p = srv.players.get(&id).unwrap();
        assert!(p.on_ground);
    }

    #[test]
    fn kill_brick_respawns() {
        use clone_core::PartKind;
        let mut w = World::baseplate();
        w.add_part_kind(
            "Kill",
            clone_core::Vec3::new(0.0, 1.0, 0.0),
            clone_core::Vec3::new(4.0, 1.0, 4.0),
            (220, 40, 40),
            true,
            PartKind::KillBrick,
            "onTouch: respawn",
        );
        let mut srv = GameServer::new(w);
        let id = srv.add_player();
        // Drop player straight onto kill brick.
        srv.players.get_mut(&id).unwrap().pos = clone_core::Vec3::new(0.0, 2.0, 0.0);
        for _ in 0..30 {
            srv.tick_once(1.0 / 60.0);
        }
        let p = srv.players.get(&id).unwrap();
        // Should have been sent back to spawn (0,5,0), not stuck inside brick.
        assert!((p.pos.x - 0.0).abs() < 5.0 && p.pos.y > 2.0);
    }

    #[test]
    fn two_players_coexist() {
        let mut srv = GameServer::new(World::baseplate());
        let a = srv.add_player();
        let b = srv.add_player();
        assert_ne!(a, b);
        for _ in 0..60 {
            srv.tick_once(1.0 / 60.0);
        }
        assert_eq!(srv.positions().len(), 2);
    }

    #[test]
    fn restore_mover_bases_undoes_live_pose() {
        use clone_core::PartKind;
        let mut w = World::baseplate();
        w.add_part_kind(
            "Mover",
            clone_core::Vec3::new(0.0, 3.0, -10.0),
            clone_core::Vec3::new(6.0, 1.0, 6.0),
            (80, 140, 230),
            true,
            PartKind::Mover,
            "tick: move 0,0,6 amplitude 6 freq 0.25",
        );
        let mut srv = GameServer::new(w);
        for _ in 0..30 {
            srv.tick_once(1.0 / 60.0);
        }
        let live = srv.world.parts.iter().find(|p| p.name == "Mover").unwrap().pos.z;
        assert!((live - -10.0).abs() > 0.1, "mover moved {live}");
        srv.restore_mover_bases();
        let base = srv.world.parts.iter().find(|p| p.name == "Mover").unwrap().pos.z;
        assert!((base - -10.0).abs() < 1e-5, "base restored {base}");
    }

    #[test]
    fn mover_platform_displaces() {
        use clone_core::PartKind;
        let mut w = World::baseplate();
        w.add_part_kind(
            "Mover",
            clone_core::Vec3::new(0.0, 3.0, -10.0),
            clone_core::Vec3::new(6.0, 1.0, 6.0),
            (80, 140, 230),
            true,
            PartKind::Mover,
            "tick: move 0,0,6 amplitude 6 freq 0.25",
        );
        let mut srv = GameServer::new(w);
        let z0 = srv.world.parts.iter().find(|p| p.name == "Mover").unwrap().pos.z;
        for _ in 0..60 {
            srv.tick_once(1.0 / 60.0);
        }
        let z1 = srv.world.parts.iter().find(|p| p.name == "Mover").unwrap().pos.z;
        assert!((z1 - z0).abs() > 0.5, "mover z {z0} -> {z1}");
    }
}
