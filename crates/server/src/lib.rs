use clone_core::{Input, PlayerState, Vec3, World};
use std::collections::HashMap;

/// Authoritative simulation. No sockets here so it stays testable.
pub struct GameServer {
    pub world: World,
    pub players: HashMap<u32, PlayerState>,
    pub inputs: HashMap<u32, Input>,
    pub tick: u64,
    next_id: u32,
}

impl GameServer {
    pub fn new(world: World) -> Self {
        Self { world, players: HashMap::new(), inputs: HashMap::new(), tick: 0, next_id: 1 }
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
        let ids: Vec<u32> = self.players.keys().copied().collect();
        for id in ids {
            let input = self.inputs.get(&id).copied().unwrap_or_default();
            if let Some(p) = self.players.get_mut(&id) {
                clone_core::step_player(&self.world, p, &input, dt);
            }
        }
    }

    pub fn positions(&self) -> Vec<(u32, Vec3)> {
        self.players.iter().map(|(id, p)| (*id, p.pos)).collect()
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
}
