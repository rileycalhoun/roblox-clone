mod render;

use clone_core::{camera::Camera, Input, Vec3, World};
use minifb::{Key, MouseButton, MouseMode, Window, WindowOptions};
use std::sync::mpsc;
use std::time::{Duration, Instant};

#[derive(PartialEq)]
enum Mode {
    Studio,
    Play,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let place_path = args.get(1).cloned().unwrap_or_else(|| "demo.rplace".to_string());
    let mut join_addr: Option<String> = None;
    let mut name = "player".to_string();
    let mut i = 2;
    while i < args.len() {
        if args[i] == "--join" && i + 1 < args.len() {
            join_addr = Some(args[i + 1].clone());
            i += 2;
        } else if args[i] == "--name" && i + 1 < args.len() {
            name = args[i + 1].clone();
            i += 2;
        } else {
            i += 1;
        }
    }

    let mut world = std::fs::read_to_string(&place_path)
        .ok()
        .and_then(|t| clone_core::load_str(&t).ok())
        .unwrap_or_else(World::baseplate);

    let mut mode = Mode::Studio;
    let mut cam = Camera::default();
    let mut target = Vec3::new(0.0, 2.0, 0.0);
    let mut selected: usize = 0;

    // Solo authoritative server (so kill-bricks + movers work offline too).
    let mut solo = clone_server::GameServer::new(world.clone());
    let solo_id = solo.add_player();

    // Net state (play-with-friends).
    let mut net: Option<NetState> = None;
    if let Some(addr) = join_addr.clone() {
        match NetState::connect(&addr, &name) {
            Ok(n) => {
                println!("joined {addr} as id={}", n.id);
                net = Some(n);
                mode = Mode::Play;
            }
            Err(e) => eprintln!("join failed: {e} (solo mode)"),
        }
    }

    let mut window = Window::new(
        "roblox-clone studio (Tab: studio/play, 1-4 asset, click/C place, X delete, S save)",
        render::W,
        render::H,
        WindowOptions::default(),
    )
    .expect("open window");

    let mut frame = render::Frame::new();
    let mut last = Instant::now();
    let mut tick: u64 = 0;
    let mut remote_players: Vec<(f32, f32, f32)> = vec![(0.0, 5.0, 0.0)];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let dt = last.elapsed().as_secs_f32().min(0.05);
        last = Instant::now();
        tick += 1;

        // --- camera orbit (both modes) ---
        if window.is_key_down(Key::Left) {
            cam.yaw -= 2.2 * dt;
        }
        if window.is_key_down(Key::Right) {
            cam.yaw += 2.2 * dt;
        }
        if window.is_key_down(Key::Up) {
            cam.pitch = (cam.pitch + 1.5 * dt).min(1.4);
        }
        if window.is_key_down(Key::Down) {
            cam.pitch = (cam.pitch - 1.5 * dt).max(-0.5);
        }
        if window.is_key_down(Key::Z) {
            cam.dist = (cam.dist - 20.0 * dt).max(5.0);
        }
        if window.is_key_down(Key::X) && mode == Mode::Studio {
            cam.dist = (cam.dist + 20.0 * dt).min(60.0);
        }

        // Tab toggles.
        if window.is_key_pressed(Key::Tab, minifb::KeyRepeat::No) {
            mode = if mode == Mode::Studio { Mode::Play } else { Mode::Studio };
            // Sync solo world when returning to studio.
            if mode == Mode::Studio {
                world = solo.world.clone();
            } else {
                solo.world = world.clone();
                solo.mover_reset();
            }
        }

        if mode == Mode::Studio {
            // Pan target.
            let (fx, fz) = cam.ground_forward();
            let rx = -fz;
            let rz = fx;
            let speed = 18.0 * dt;
            if window.is_key_down(Key::W) {
                target.x += fx * speed;
                target.z += fz * speed;
            }
            if window.is_key_down(Key::S) {
                target.x -= fx * speed;
                target.z -= fz * speed;
            }
            if window.is_key_down(Key::A) {
                target.x -= rx * speed;
                target.z -= rz * speed;
            }
            if window.is_key_down(Key::D) {
                target.x += rx * speed;
                target.z += rz * speed;
            }
            if window.is_key_down(Key::R) {
                target.y += speed;
            }
            if window.is_key_down(Key::F) {
                target.y -= speed;
            }
            for (n, k) in [Key::Key1, Key::Key2, Key::Key3, Key::Key4].iter().enumerate() {
                if window.is_key_pressed(*k, minifb::KeyRepeat::No) {
                    selected = n;
                }
            }
            // Place asset: click or C.
            let clicked = window.get_mouse_down(MouseButton::Left);
            static mut LAST_CLICK: bool = false;
            let edge = clicked && unsafe { !LAST_CLICK };
            unsafe {
                LAST_CLICK = clicked;
            }
            if edge || window.is_key_pressed(Key::C, minifb::KeyRepeat::No) {
                let keys = ["block", "spawn", "kill", "mover"];
                let (fx, fz) = cam.ground_forward();
                let pos = Vec3::new(target.x + fx * 8.0, target.y, target.z + fz * 8.0);
                let snapped = Vec3::new(pos.x.round(), (pos.y + 0.5).round() - 0.5, pos.z.round());
                if clone_core::assets::instantiate(&mut world, keys[selected], snapped).is_some() {
                    solo.world = world.clone();
                    solo.mover_reset();
                }
            }
            if window.is_key_pressed(Key::X, minifb::KeyRepeat::No) {
                // Delete nearest part to aim.
                let (fx, fz) = cam.ground_forward();
                let aim = Vec3::new(target.x + fx * 8.0, target.y, target.z + fz * 8.0);
                if let Some(id) = nearest_part(&world, aim) {
                    world.remove_part(id);
                    solo.world = world.clone();
                    solo.mover_reset();
                }
            }
            if window.is_key_pressed(Key::S, minifb::KeyRepeat::No) {
                std::fs::write(&place_path, clone_core::save_str(&world)).expect("save");
                println!("saved {place_path}");
            }
            let eye = cam.eye(target);
            let players = vec![(world.spawn.pos.x, world.spawn.pos.y, world.spawn.pos.z)];
            render::render(&mut frame, &world, &players, eye, cam.yaw, cam.pitch);
            let keys = ["block", "spawn", "kill", "mover"];
            window.set_title(&format!(
                "STUDIO {place_path} | asset={} | parts={} | Tab=play S=save click/C=place X=del",
                keys[selected],
                world.parts.len()
            ));
        } else {
            // --- Play mode: input relative to camera ---
            let (fx, fz) = cam.ground_forward();
            let rx = -fz;
            let rz = fx;
            let (mut fwd, mut side) = (0.0f32, 0.0f32);
            if window.is_key_down(Key::W) {
                fwd += 1.0;
            }
            if window.is_key_down(Key::S) {
                fwd -= 1.0;
            }
            if window.is_key_down(Key::A) {
                side -= 1.0;
            }
            if window.is_key_down(Key::D) {
                side += 1.0;
            }
            // Remap camera-relative to world input (fwd=+Z, side=+X).
            let wf = fwd * fz + side * rz;
            let ws = fwd * fx + side * rx;
            let jump = window.is_key_down(Key::Space);
            // Note: physics Input uses fwd->Z, side->X; camera yaw=0 faces -Z.
            // Our ground_forward already points -Z at yaw 0, so map directly.
            let solo_input = Input { fwd: wf, side: ws, jump };

            if window.is_key_pressed(Key::R, minifb::KeyRepeat::No) {
                if let Some(n) = net.as_mut() {
                    // respawn handled server-side via kill; just jump
                    let _ = n;
                } else {
                    if let Some(p) = solo.players.get_mut(&solo_id) {
                        p.pos = solo.world.spawn.pos;
                        p.vel = Vec3::ZERO;
                    }
                }
            }

            if let Some(n) = net.as_mut() {
                n.send(Input { fwd: wf, side: ws, jump }, tick);
                n.poll(&mut remote_players);
                // Camera follows first remote render pos (own id preferred).
                let me = remote_players
                    .iter()
                    .find(|_| true)
                    .copied()
                    .unwrap_or((0.0, 5.0, 0.0));
                let t = Vec3::new(me.0, me.1 + 2.0, me.2);
                let eye = cam.eye(t);
                render::render(&mut frame, &world, &remote_players, eye, cam.yaw, cam.pitch);
                window.set_title(&format!(
                    "PLAY online {} players={} WASD+space Tab=studio",
                    join_addr.clone().unwrap_or_default(),
                    remote_players.len()
                ));
            } else {
                solo.set_input(solo_id, solo_input);
                solo.tick_once(dt.min(1.0 / 30.0));
                let me = solo.players.get(&solo_id).unwrap().pos;
                let t = Vec3::new(me.x, me.y + 2.0, me.z);
                let eye = cam.eye(t);
                let ps: Vec<(f32, f32, f32)> =
                    solo.positions().iter().map(|(_, v)| (v.x, v.y, v.z)).collect();
                render::render(&mut frame, &solo.world, &ps, eye, cam.yaw, cam.pitch);
                window.set_title(&format!(
                    "PLAY solo pos={:.1},{:.1},{:.1} WASD+space R=respawn Tab=studio",
                    me.x, me.y, me.z
                ));
            }
        }

        // Mouse drag orbits too.
        if let Some((mx, my)) = window.get_mouse_pos(MouseMode::Clamp) {
            static mut LX: f32 = -1.0;
            static mut LY: f32 = -1.0;
            unsafe {
                if window.get_mouse_down(MouseButton::Right) {
                    if LX >= 0.0 {
                        cam.yaw += (mx - LX) * 0.01;
                        cam.pitch = (cam.pitch + (my - LY) * 0.01).clamp(-0.5, 1.4);
                    }
                }
                LX = mx;
                LY = my;
            }
        }

        window.update_with_buffer(&frame.buf, render::W, render::H).unwrap();
        std::thread::sleep(Duration::from_millis(8));
    }
}

fn nearest_part(world: &World, to: Vec3) -> Option<u32> {
    let mut best: Option<(u32, f32)> = None;
    for p in &world.parts {
        let d = p.pos.sub(to);
        let dd = d.x * d.x + d.y * d.y + d.z * d.z;
        if best.map(|(_, bd)| dd < bd).unwrap_or(true) {
            best = Some((p.id, dd));
        }
    }
    best.map(|(id, _)| id)
}

struct NetState {
    id: u32,
    tx: std::sync::mpsc::Sender<clone_core::Input>,
    rx: mpsc::Receiver<Vec<(f32, f32, f32)>>,
}

impl NetState {
    fn connect(addr: &str, name: &str) -> std::io::Result<Self> {
        use std::io::{BufRead, BufReader, Write};
        use std::net::TcpStream;
        let stream = TcpStream::connect(addr)?;
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        let mut w = stream.try_clone()?;
        let mut r = BufReader::new(stream);
        let hello = clone_core::net::encode_client(&clone_core::net::ClientMsg::Hello {
            name: name.to_string(),
        });
        w.write_all(hello.as_bytes())?;
        let mut line = String::new();
        r.read_line(&mut line)?;
        let id = match clone_core::net::decode_server(&line) {
            Some(clone_core::net::ServerMsg::Welcome { id }) => id,
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "bad welcome",
                ))
            }
        };
        // IO thread: owns sockets, main thread sends inputs + receives player lists.
        let (itx, irx) = mpsc::channel::<clone_core::Input>();
        let (otx, orx) = mpsc::channel::<Vec<(f32, f32, f32)>>();
        std::thread::spawn(move || {
            let mut writer = w;
            let mut reader = r;
            let mut seq = 0u32;
            let mut tick = 0u64;
            loop {
                // Drain latest input (non-blocking).
                let mut cur: Option<clone_core::Input> = None;
                while let Ok(inp) = irx.try_recv() {
                    cur = Some(inp);
                }
                if let Some(inp) = cur {
                    seq += 1;
                    tick += 1;
                    let msg = clone_core::net::ClientMsg::Input {
                        seq,
                        tick,
                        fwd: inp.fwd,
                        side: inp.side,
                        jump: inp.jump,
                    };
                    if writer.write_all(clone_core::net::encode_client(&msg).as_bytes()).is_err() {
                        break;
                    }
                }
                // Try one snapshot line with short timeout.
                reader
                    .get_ref()
                    .set_read_timeout(Some(Duration::from_millis(5)))
                    .ok();
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(0) => break,
                    Ok(_) => {
                        if let Some(clone_core::net::ServerMsg::Snapshot { players, .. }) =
                            clone_core::net::decode_server(&line)
                        {
                            let list: Vec<(f32, f32, f32)> =
                                players.iter().map(|(_, v)| (v.x, v.y, v.z)).collect();
                            let _ = otx.send(list);
                        }
                    }
                    Err(_) => {}
                }
                std::thread::sleep(Duration::from_millis(16));
            }
        });
        Ok(Self { id, tx: itx, rx: orx })
    }

    fn send(&mut self, input: Input, _tick: u64) {
        let _ = self.tx.send(input);
    }

    fn poll(&mut self, out: &mut Vec<(f32, f32, f32)>) {
        while let Ok(list) = self.rx.try_recv() {
            *out = list;
        }
    }
}
