
You are tasked with building an MVP of a feature-complete Roblox clone, with a 'Roblox Studio' equivalent, 
a way to play the created games, and a way for users to host these games on remote servers.

You will be using the device you are currently on for this test, which means you will be working on a:
MacBook Pro (M5, 13-inch, 16GB RAM, 1TB storage)

Your project must be written entirely in Rust. Do not use any other language.
Use as little outside libraries as possible. Be self-sufficient: if you need something, make it.

You are given a task list below, update it as you complete tasks and add more as you plan out your process.
Update your OpenCode task list as well so the user sees what you are working on without opening this file.

Keep your TODO list and goal up to date with the latest information.
Create a repository with `gh` and push as you complete tasks.

YOUR TASKS:
 - [x] Decide what 'MVP' means. What's the scope here?
   -> Decided: MVP v0.1 = headless Rust-std-only loop. Core (Vec3/Part/World/AABB physics/.rplace/net/PPM preview),
      Studio CLI (new/list/add/move/delete/preview/playtest), Player (NetClient + LocalPlay),
      Server (authoritative 60Hz sim + 20Hz TCP snapshots). Deferred v0.2: winit/wgpu 3D, camera, scripting.
 - [x] Scaffold Rust workspace (core/server/player/studio, zero deps) + cargo test green (7 tests)
 - [x] Verify end-to-end: studio save/list/playtest/preview + server<->player snapshots (y 5->2.5 landing, 16 u/s)
 - [ ] v0.2 playable demo (user can open Studio, use assets, script game, play with friends):
   - [x] Studio window opens with 3D viewport + asset panel + save/load (.rplace)
     -> clone_app (minifb, 1 dep): Tab studio/play, 1-4 assets, click/C place, X delete, S save, arrows orbit
   - [x] Assets catalog to create games (blocks, spawn, kill-brick, moving platform)
     -> clone_core::assets CATALOG(4) + studio assets/asset-add/demo-place; verified list + demo.rplace
   - [x] Script games (Lua-subset in Rust: onJoin/onTouch/tick)
     -> clone_core::script parse + mover_offset + kill detect (3 tests); server applies movers + touch-respawn; studio script-check OK
   - [x] Play game solo (third-person WASD+jump+camera)
     -> clone_app play mode: GameServer solo tick + Camera follow + WASD/space/R; builds + launches 3s no crash
   - [x] Play game with friends (server browser/join, 2+ players replicated 20Hz)
     -> clone_app --join addr + NetState thread; verified alice(id2)+bob(id1) both see 2-player snapshots 20Hz
   - [ ] Verify end-to-end playable + push with gh
