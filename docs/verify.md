# Playable-demo verification audit (2026-10-07)

Objective: MVP Roblox clone — Studio equivalent, play created games, host on
remote servers. Rust-only (one window dep: `minifb`), MacBook M5.

## Criterion → evidence

| # | Requirement | Evidence | Status |
|---|-------------|----------|--------|
| 1 | Decide MVP scope, keep task/todo/goal lists current | `AGENTS.md` task list, OpenCode todos, goal history | ✅ done |
| 2 | Rust workspace, minimal deps, `gh` repo + pushes | `Cargo.toml` workspace (core/server/player/studio/app), `minifb` only new dep, https://github.com/rileycalhoun/roblox-clone (`git log` shows per-task pushes) | ✅ done |
| 3 | Studio: open, assets, save/load | Live window photographed open (`docs-live-studio.png`: HUD `parts=5`, world renders); `clone_studio assets/asset-add/demo-place/list`; human placed 22 blocks + saved (file 5→27 parts) | ✅ done (human) |
| 4 | Scripting (Lua-subset onJoin/onTouch/tick) | `clone_core::script` unit tests; server applies movers + touch-respawn (unit tests); `studio script-check`; live mover displacement in soak (`mover_z=-4.2`) | ✅ done |
| 5 | Solo play (third-person WASD+jump+camera) | Play branch code + `--soak` (scripted W+Jump through real `GameServer`) + `--screenshot-play` follow-cam render (`docs-user-level.png`, visually confirmed) | ✅ code/headless; ⚠️ human in-window run unconfirmed |
| 6 | Play with friends (join, 2+ replicated 20 Hz) | 2× `clone_player` vs real server (debug + release, base + user levels): 90 snapshots each, both see ids 1+2; GUI pump has ephemeral-port handshake test (`PumpClient`) | ✅ protocol; ⚠️ human GUI session unconfirmed |
| 7 | Remote hosting | `clone_server 0.0.0.0:8772` + README Hosting section; release profile built and replicated end-to-end | ✅ done (localhost; remote host = same binary) |
| 8 | Suite green, no warnings | `cargo test`: 22 passed (13 core, 5 server, 1 player, 3 app), `cargo build` zero warnings | ✅ done |

## Reproduce

```sh
cargo test
./target/debug/clone_studio demo-place demo.rplace
./target/debug/clone_server 127.0.0.1:8772 demo.rplace &
./target/debug/clone_app demo.rplace                       # Tab studio/play, 1-4 + click, WASD+space
./target/debug/clone_app demo.rplace --join 127.0.0.1:8772 --name you
```

## Open item

Human in-window play (Tab → WASD + jump, optionally `--join` with a friend).
Blocked for automation: macOS denies synthetic keystrokes (error 1002).
