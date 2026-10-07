# roblox-clone (MVP v0.1)

Rust-only Roblox clone. Zero external dependencies (std only).

## MVP scope (decided)

v0.1 headless MVP proves the full loop without graphics deps:
- **Core**: `Vec3`, `Part`/`World`, AABB character controller (gravity, jump, void reset),
  `.rplace` text place format (hand-rolled, no serde), line TCP protocol, PPM side-view renderer.
- **Studio equivalent** (`clone_studio`): `new/list/add/move/delete/save-default/preview/playtest`.
- **Player** (`clone_player` + `clone_player::LocalPlay`): TCP client + offline sim.
- **Remote host** (`clone_server` + `clone_server::GameServer`): authoritative 60 Hz sim,
  20 Hz snapshots, per-client threads, `clone_server <bind> [place]`.

Deferred to v0.2: real-time 3D window (winit/wgpu, 2 deps max), camera, Lua-subset scripting,
persistence/avatars/catalog.

## Quickstart

```sh
cargo test
./target/debug/clone_studio save-default demo.rplace
./target/debug/clone_studio list demo.rplace
./target/debug/clone_studio playtest demo.rplace 180
./target/debug/clone_studio preview demo.rplace preview.ppm

# remote host + play
./target/debug/clone_server 127.0.0.1:8765 demo.rplace &
./target/debug/clone_player 127.0.0.1:8765 tester
```

Verified: client falls from spawn y=5 to ground y=2.5 then moves at 16 u/s, snapshots at 20 Hz.

## v0.2 playable demo

```sh
cargo test  # 22 tests green (13 core, 5 server, 1 player pump, 3 app)
./target/debug/clone_studio demo-place demo.rplace
./target/debug/clone_studio assets
./target/debug/clone_server 127.0.0.1:8772 demo.rplace &
./target/debug/clone_app demo.rplace                       # Tab studio/play, 1-4 + click place, WASD+space
./target/debug/clone_app demo.rplace --join 127.0.0.1:8772 --name you   # play with friends
```

Scripting (`onJoin/onTouch/tick`): `clone_studio script-check demo.rplace 3`.

Headless checks: `clone_app demo.rplace --screenshot viewport.ppm`,
`clone_app demo.rplace --screenshot-play play.ppm 240`,
`clone_app demo.rplace --soak 550`.

## Hosting a game on a remote server

One TCP port, no auth in the MVP (anyone with the address can join):

```sh
cargo build --release
./target/release/clone_studio demo-place mygame.rplace
./target/release/clone_server 0.0.0.0:8772 mygame.rplace   # leave running (tmux/nohup/systemd)
```

Friends join with `./target/debug/clone_app mygame.rplace --join YOUR_HOST:8772 --name them`.
The server is authoritative (60 Hz sim, 20 Hz snapshots); scripts (movers,
kill bricks) run server-side so every client sees the same game.

## Layout

- `crates/core`: math/model/physics/place/net/preview (std only)
- `crates/server`: `GameServer` lib + TCP dedicated server
- `crates/player`: `NetClient` + `LocalPlay` + demo binary
- `crates/studio`: headless editor CLI
