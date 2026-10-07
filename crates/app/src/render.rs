//! Software 3D renderer to a u32 framebuffer (no graphics deps beyond minifb window).
//! Perspective orbit camera, wireframe + filled top faces, players as markers.

use clone_core::{Vec3, World};

pub const W: usize = 800;
pub const H: usize = 600;

pub struct Frame {
    pub buf: Vec<u32>,
}

impl Frame {
    pub fn new() -> Self {
        Self { buf: vec![0x181826; W * H] }
    }

    pub fn clear(&mut self) {
        self.buf.fill(0x181826);
    }

    pub fn put(&mut self, x: i32, y: i32, c: u32) {
        if x >= 0 && y >= 0 && (x as usize) < W && (y as usize) < H {
            self.buf[y as usize * W + x as usize] = c;
        }
    }

    pub fn line(&mut self, mut x0: i32, mut y0: i32, x1: i32, y1: i32, c: u32) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            self.put(x0, y0, c);
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    /// Filled convex quad (assumes screen-space convex, e.g. top face).
    pub fn quad(&mut self, pts: [(i32, i32); 4], c: u32) {
        let min_y = pts.iter().map(|p| p.1).min().unwrap_or(0).max(0);
        let max_y = pts.iter().map(|p| p.1).max().unwrap_or(0).min(H as i32 - 1);
        for y in min_y..=max_y {
            let mut xs = Vec::new();
            for i in 0..4 {
                let (x0, y0) = pts[i];
                let (x1, y1) = pts[(i + 1) % 4];
                if (y0 <= y && y < y1) || (y1 <= y && y < y0) {
                    let t = (y - y0) as f32 / (y1 - y0) as f32;
                    xs.push((x0 as f32 + t * (x1 - x0) as f32) as i32);
                }
            }
            xs.sort();
            if xs.len() >= 2 {
                let (a, b) = (xs[0].max(0), xs[xs.len() - 1].min(W as i32 - 1));
                for x in a..=b {
                    self.put(x, y, c);
                }
            }
        }
    }
}

pub fn rgb(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

/// Project world point to screen. Returns (x, y, depth).
pub fn project(p: Vec3, eye: Vec3, yaw: f32, pitch: f32) -> Option<(i32, i32, f32)> {
    let d = p.sub(eye);
    // Inverse orbit rotation: yaw then pitch.
    let (sy, cy) = (-yaw).sin_cos();
    let x1 = d.x * cy + d.z * sy;
    let z1 = -d.x * sy + d.z * cy;
    // Inverse of eye placement (eye = Ry(yaw) * Rx(-pitch) * (0,0,dist)):
    // view rotation is Rx(+pitch) * Ry(-yaw).
    let (sp, cp) = pitch.sin_cos();
    let y2 = d.y * cp - z1 * sp;
    let z2 = d.y * sp + z1 * cp;
    // Camera looks along -Z.
    let fwd = -z2;
    if fwd < 0.3 {
        return None;
    }
    let f = 520.0;
    let sx = (W as f32 * 0.5 + x1 / fwd * f) as i32;
    let sy = (H as f32 * 0.5 - y2 / fwd * f) as i32;
    Some((sx, sy, fwd))
}

fn box_corners(pos: Vec3, size: Vec3) -> [Vec3; 8] {
    let hx = size.x * 0.5;
    let hy = size.y * 0.5;
    let hz = size.z * 0.5;
    [
        Vec3::new(pos.x - hx, pos.y - hy, pos.z - hz),
        Vec3::new(pos.x + hx, pos.y - hy, pos.z - hz),
        Vec3::new(pos.x + hx, pos.y - hy, pos.z + hz),
        Vec3::new(pos.x - hx, pos.y - hy, pos.z + hz),
        Vec3::new(pos.x - hx, pos.y + hy, pos.z - hz),
        Vec3::new(pos.x + hx, pos.y + hy, pos.z - hz),
        Vec3::new(pos.x + hx, pos.y + hy, pos.z + hz),
        Vec3::new(pos.x - hx, pos.y + hy, pos.z + hz),
    ]
}

const EDGES: [(usize, usize); 12] = [
    (0, 1), (1, 2), (2, 3), (3, 0), // bottom
    (4, 5), (5, 6), (6, 7), (7, 4), // top
    (0, 4), (1, 5), (2, 6), (3, 7), // sides
];

/// Render world + players (x,y,z list). Players drawn as yellow boxes.
pub fn render(frame: &mut Frame, world: &World, players: &[(f32, f32, f32)], eye: Vec3, yaw: f32, pitch: f32) {
    frame.clear();
    // Ground grid.
    for i in -8..=8 {
        let a = Vec3::new(i as f32 * 4.0, 0.0, -32.0);
        let b = Vec3::new(i as f32 * 4.0, 0.0, 32.0);
        if let (Some((x0, y0, _)), Some((x1, y1, _))) =
            (project(a, eye, yaw, pitch), project(b, eye, yaw, pitch))
        {
            frame.line(x0, y0, x1, y1, 0x2a2a3a);
        }
        let a = Vec3::new(-32.0, 0.0, i as f32 * 4.0);
        let b = Vec3::new(32.0, 0.0, i as f32 * 4.0);
        if let (Some((x0, y0, _)), Some((x1, y1, _))) =
            (project(a, eye, yaw, pitch), project(b, eye, yaw, pitch))
        {
            frame.line(x0, y0, x1, y1, 0x2a2a3a);
        }
    }
    // Parts, far-to-near by distance to eye.
    let mut order: Vec<usize> = (0..world.parts.len()).collect();
    order.sort_by(|&a, &b| {
        let da = dist2(world.parts[a].pos, eye);
        let db = dist2(world.parts[b].pos, eye);
        db.partial_cmp(&da).unwrap()
    });
    for &i in &order {
        let p = &world.parts[i];
        let corners = box_corners(p.pos, p.size);
        // Per-corner projection: faces/edges with a corner behind the near
        // plane are skipped individually, so big boxes (baseplate) still draw
        // their visible faces when the camera is over them.
        let scr: Vec<Option<(i32, i32, f32)>> =
            corners.iter().map(|c| project(*c, eye, yaw, pitch)).collect();
        let base = rgb(p.color.0, p.color.1, p.color.2);
        // Top face fill.
        if let [Some(t0), Some(t1), Some(t2), Some(t3)] = [scr[4], scr[5], scr[6], scr[7]] {
            frame.quad([(t0.0, t0.1), (t1.0, t1.1), (t2.0, t2.1), (t3.0, t3.1)], base);
        }
        for (a, b) in EDGES {
            if let (Some(pa), Some(pb)) = (scr[a], scr[b]) {
                frame.line(pa.0, pa.1, pb.0, pb.1, 0x101018);
            }
        }
    }
    // Players.
    for (x, y, z) in players {
        let pos = Vec3::new(*x, *y, *z);
        let size = Vec3::new(2.0, 5.0, 2.0);
        let corners = box_corners(pos, size);
        let scr: Vec<Option<(i32, i32, f32)>> =
            corners.iter().map(|c| project(*c, eye, yaw, pitch)).collect();
        if let [Some(t0), Some(t1), Some(t2), Some(t3)] = [scr[4], scr[5], scr[6], scr[7]] {
            frame.quad([(t0.0, t0.1), (t1.0, t1.1), (t2.0, t2.1), (t3.0, t3.1)], 0xffe63a);
        }
        for (a, b) in EDGES {
            if let (Some(pa), Some(pb)) = (scr[a], scr[b]) {
                frame.line(pa.0, pa.1, pb.0, pb.1, 0x6a5a00);
            }
        }
    }
}

fn dist2(a: Vec3, b: Vec3) -> f32 {
    let d = a.sub(b);
    d.x * d.x + d.y * d.y + d.z * d.z
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orbit_target_projects_to_screen_center() {
        use clone_core::camera::Camera;
        for (yaw, pitch) in [(0.0, 0.35), (0.6, 0.5), (2.0, 0.2)] {
            let cam = Camera { yaw, pitch, dist: 22.0 };
            let target = Vec3::new(0.0, 1.0, -2.0);
            let (sx, sy, depth) = project(target, cam.eye(target), yaw, pitch).expect("in front");
            assert!((sx - W as i32 / 2).abs() < 3, "yaw={yaw} sx={sx}");
            assert!((sy - H as i32 / 2).abs() < 3, "pitch={pitch} sy={sy}");
            assert!(depth > 0.0);
        }
    }

    #[test]
    fn viewport_renders_world_headless() {
        let mut world = World::baseplate();
        world.add_part(
            "Tower",
            Vec3::new(0.0, 5.0, 10.0),
            Vec3::new(4.0, 10.0, 4.0),
            (200, 50, 50),
            true,
        );
        let mut frame = Frame::new();
        let eye = Vec3::new(0.0, 8.0, 24.0);
        render(&mut frame, &world, &[(0.0, 5.0, 0.0)], eye, 0.0, 0.2);
        let bg = 0x181826;
        let painted = frame.buf.iter().filter(|&&c| c != bg).count();
        assert!(painted > 1000, "viewport drew {painted} px");
    }

    #[test]
    fn close_camera_still_draws_partial_boxes() {
        // Camera right next to the tower: some corners are behind the near
        // plane. The box must still draw its visible edges, not vanish.
        let mut world = World::empty();
        world.add_part(
            "Wall",
            Vec3::new(0.0, 2.0, 0.0),
            Vec3::new(4.0, 4.0, 1.0),
            (200, 50, 50),
            true,
        );
        let mut frame = Frame::new();
        let eye = Vec3::new(0.0, 2.0, 3.0);
        render(&mut frame, &world, &[], eye, 0.0, 0.0);
        let edge = frame.buf.iter().filter(|&&c| c == 0x101018).count();
        assert!(edge > 20, "visible edges drew {edge} px");
    }
}
