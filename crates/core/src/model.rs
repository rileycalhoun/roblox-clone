//! Data model: parts, spawn, world.

use crate::math::Vec3;

#[derive(Debug, Clone, PartialEq)]
pub struct Part {
    pub id: u32,
    pub name: String,
    pub pos: Vec3,
    pub size: Vec3,
    pub color: (u8, u8, u8),
    pub anchored: bool,
}

impl Part {
    pub fn min(&self) -> Vec3 {
        Vec3::new(
            self.pos.x - self.size.x * 0.5,
            self.pos.y - self.size.y * 0.5,
            self.pos.z - self.size.z * 0.5,
        )
    }
    pub fn max(&self) -> Vec3 {
        Vec3::new(
            self.pos.x + self.size.x * 0.5,
            self.pos.y + self.size.y * 0.5,
            self.pos.z + self.size.z * 0.5,
        )
    }

    pub fn overlaps(&self, min: Vec3, max: Vec3) -> bool {
        let a0 = self.min();
        let a1 = self.max();
        a0.x < max.x && a1.x > min.x && a0.y < max.y && a1.y > min.y && a0.z < max.z && a1.z > min.z
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Spawn {
    pub pos: Vec3,
}

#[derive(Debug, Clone)]
pub struct World {
    pub parts: Vec<Part>,
    pub spawn: Spawn,
    pub next_id: u32,
}

impl World {
    pub fn empty() -> Self {
        Self {
            parts: Vec::new(),
            spawn: Spawn { pos: Vec3::new(0.0, 5.0, 0.0) },
            next_id: 1,
        }
    }

    pub fn baseplate() -> Self {
        let mut w = Self::empty();
        w.add_part(
            "Baseplate",
            Vec3::new(0.0, -0.5, 0.0),
            Vec3::new(64.0, 1.0, 64.0),
            (100, 150, 100),
            true,
        );
        w
    }

    pub fn add_part(
        &mut self,
        name: &str,
        pos: Vec3,
        size: Vec3,
        color: (u8, u8, u8),
        anchored: bool,
    ) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.parts.push(Part {
            id,
            name: name.to_string(),
            pos,
            size,
            color,
            anchored,
        });
        id
    }

    pub fn remove_part(&mut self, id: u32) -> bool {
        let n = self.parts.len();
        self.parts.retain(|p| p.id != id);
        self.parts.len() != n
    }

    pub fn get(&self, id: u32) -> Option<&Part> {
        self.parts.iter().find(|p| p.id == id)
    }

    pub fn get_mut(&mut self, id: u32) -> Option<&mut Part> {
        self.parts.iter_mut().find(|p| p.id == id)
    }

    /// All anchored parts overlapping the given AABB.
    pub fn colliders(&self, min: Vec3, max: Vec3) -> impl Iterator<Item = &Part> {
        self.parts
            .iter()
            .filter(move |p| p.anchored && p.overlaps(min, max))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_remove() {
        let mut w = World::empty();
        let id = w.add_part("P", Vec3::new(0.0, 1.0, 0.0), Vec3::new(4.0, 1.0, 4.0), (255, 0, 0), true);
        assert!(w.get(id).is_some());
        assert!(w.remove_part(id));
        assert!(w.get(id).is_none());
    }
}
