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

## Layout

- `crates/core`: math/model/physics/place/net/preview (std only)
- `crates/server`: `GameServer` lib + TCP dedicated server
- `crates/player`: `NetClient` + `LocalPlay` + demo binary
- `crates/studio`: headless editor CLI
