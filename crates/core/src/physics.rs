//! Axis-separated AABB character controller. Headless, deterministic.

use crate::math::Vec3;
use crate::model::World;

pub const GRAVITY: f32 = -22.0;
pub const MOVE_SPEED: f32 = 16.0;
pub const JUMP_VEL: f32 = 11.0;

#[derive(Debug, Clone, Copy)]
pub struct Input {
    /// -1..1 forward (maps to -Z/+Z for MVP, camera fixed)
    pub fwd: f32,
    /// -1..1 strafe (maps to X)
    pub side: f32,
    pub jump: bool,
}

impl Default for Input {
    fn default() -> Self {
        Self { fwd: 0.0, side: 0.0, jump: false }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PlayerState {
    pub pos: Vec3,
    pub vel: Vec3,
    pub on_ground: bool,
    /// Half extents of player box. Default like Roblox R15-ish: 1 x 2.5 x 1 full.
    pub half: Vec3,
}

impl PlayerState {
    pub fn at(pos: Vec3) -> Self {
        Self {
            pos,
            vel: Vec3::ZERO,
            on_ground: false,
            half: Vec3::new(1.0, 2.5, 1.0),
        }
    }

    pub fn min(&self) -> Vec3 {
        self.pos.sub(self.half)
    }
    pub fn max(&self) -> Vec3 {
        self.pos.add(self.half)
    }
}

fn collides(world: &World, pos: Vec3, half: Vec3) -> bool {
    let min = pos.sub(half);
    let max = pos.add(half);
    world.colliders(min, max).next().is_some()
}

pub fn step_player(world: &World, p: &mut PlayerState, input: &Input, dt: f32) {
    let fwd = input.fwd.clamp(-1.0, 1.0);
    let side = input.side.clamp(-1.0, 1.0);

    // Horizontal velocity directly set (arcade controller, MVP).
    p.vel.x = side * MOVE_SPEED;
    p.vel.z = fwd * MOVE_SPEED;
    // Gravity.
    p.vel.y += GRAVITY * dt;
    if p.vel.y < -40.0 {
        p.vel.y = -40.0;
    }
    if input.jump && p.on_ground {
        p.vel.y = JUMP_VEL;
        p.on_ground = false;
    }

    // Move axis by axis, revert colliding axis.
    let mut next = p.pos;

    // X
    let try_x = Vec3::new(next.x + p.vel.x * dt, next.y, next.z);
    if collides(world, try_x, p.half) {
        p.vel.x = 0.0;
    } else {
        next.x = try_x.x;
    }
    // Z
    let try_z = Vec3::new(next.x, next.y, next.z + p.vel.z * dt);
    if collides(world, try_z, p.half) {
        p.vel.z = 0.0;
    } else {
        next.z = try_z.z;
    }
    // Y
    let try_y = Vec3::new(next.x, next.y + p.vel.y * dt, next.z);
    if collides(world, try_y, p.half) {
        if p.vel.y <= 0.0 {
            p.on_ground = true;
        }
        p.vel.y = 0.0;
    } else {
        next.y = try_y.y;
        // grounded probe: 0.05 below
        let probe = Vec3::new(next.x, next.y - 0.06, next.z);
        p.on_ground = collides(world, probe, p.half);
    }

    p.pos = next;

    // Void reset.
    if p.pos.y < -50.0 {
        p.pos = Vec3::new(0.0, 10.0, 0.0);
        p.vel = Vec3::ZERO;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn falls_and_lands_on_baseplate() {
        let world = World::baseplate();
        let mut p = PlayerState::at(Vec3::new(0.0, 10.0, 0.0));
        let input = Input::default();
        for _ in 0..240 {
            step_player(&world, &mut p, &input, 1.0 / 60.0);
        }
        assert!(p.on_ground, "player should land, pos={:?}", p.pos);
        // Top of baseplate is y=0, player half height 2.5 -> center ~2.5
        assert!((p.pos.y - 2.5).abs() < 0.3, "pos.y={}", p.pos.y);
    }

    #[test]
    fn jump_reaches_air() {
        let world = World::baseplate();
        let mut p = PlayerState::at(Vec3::new(0.0, 2.6, 0.0));
        p.on_ground = true;
        let jump = Input { fwd: 0.0, side: 0.0, jump: true };
        step_player(&world, &mut p, &jump, 1.0 / 60.0);
        assert!(!p.on_ground || p.vel.y > 0.0);
    }
}
