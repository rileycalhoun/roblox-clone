//! Std-only software preview renderer: orthographic top/front projection to PPM.
//! Proves the play path visually without any graphics dependency.

use crate::model::World;

pub struct Ppm {
    pub w: u32,
    pub h: u32,
    pub px: Vec<(u8, u8, u8)>,
}

impl Ppm {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = format!("P6\n{} {}\n255\n", self.w, self.h).into_bytes();
        for (r, g, b) in &self.px {
            out.push(*r);
            out.push(*g);
            out.push(*b);
        }
        out
    }
}

/// Render world + player dots from a side view (x -> right, y -> up).
pub fn render_side(world: &World, players: &[(f32, f32, f32)], w: u32, h: u32) -> Ppm {
    let mut px = vec![(24u8, 24u8, 32u8); (w * h) as usize];
    // World bounds guess: x in [-40,40], y in [0,30].
    let project = |x: f32, y: f32| -> Option<(u32, u32)> {
        if !( -40.0..=40.0).contains(&x) || !(0.0..=30.0).contains(&y) {
            return None;
        }
        let sx = ((x + 40.0) / 80.0 * (w as f32 - 1.0)) as u32;
        let sy = ((1.0 - y / 30.0) * (h as f32 - 1.0)) as u32;
        Some((sx.min(w - 1), sy.min(h - 1)))
    };
    for p in &world.parts {
        let x0 = p.pos.x - p.size.x * 0.5;
        let x1 = p.pos.x + p.size.x * 0.5;
        let y0 = p.pos.y - p.size.y * 0.5;
        let y1 = p.pos.y + p.size.y * 0.5;
        if let (Some((ax, ay0)), Some((bx, ay1))) = (project(x0, y1), project(x1, y0)) {
            let (xa, xb) = (ax.min(bx), ax.max(bx));
            let (ya, yb) = (ay0.min(ay1), ay0.max(ay1));
            for yy in ya..=yb {
                for xx in xa..=xb {
                    px[(yy * w + xx) as usize] = p.color;
                }
            }
        }
    }
    for (x, y, _z) in players {
        if let Some((sx, sy)) = project(*x, *y) {
            for dy in -2i32..=2 {
                for dx in -2i32..=2 {
                    let xx = sx as i32 + dx;
                    let yy = sy as i32 + dy;
                    if xx >= 0 && yy >= 0 && (xx as u32) < w && (yy as u32) < h {
                        px[(yy as u32 * w + xx as u32) as usize] = (255, 255, 0);
                    }
                }
            }
        }
    }
    Ppm { w, h, px }
}
