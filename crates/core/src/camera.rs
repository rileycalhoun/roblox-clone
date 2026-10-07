//! Third-person orbit camera math (std only).

use crate::math::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub yaw: f32,
    pub pitch: f32,
    pub dist: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self { yaw: 0.0, pitch: 0.35, dist: 12.0 }
    }
}

impl Camera {
    /// Eye position looking at target.
    pub fn eye(&self, target: Vec3) -> Vec3 {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        Vec3::new(
            target.x + sy * cp * self.dist,
            target.y + sp * self.dist,
            target.z + cy * cp * self.dist,
        )
    }

    /// Forward direction on the ground plane from yaw (for WASD).
    pub fn ground_forward(&self) -> (f32, f32) {
        // yaw=0 faces -Z like classic default.
        (-self.yaw.sin(), -self.yaw.cos())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eye_behind_target() {
        let c = Camera::default();
        let eye = c.eye(Vec3::new(0.0, 2.5, 0.0));
        assert!(eye.y > 2.5);
    }
}
