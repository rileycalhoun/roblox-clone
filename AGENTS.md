
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
     -> 2026-10-07: 19 tests green (13 core, 4 server, 2 app incl. orbit-center);
        demo.rplace list/script-check/playtest grounded OK; alice+bob 90 snapshots each see ids 1+2;
        clone_app --screenshot renders all 4 assets+player (demo-viewport.png, visually confirmed);
        --soak 550: grounded z=-23.5, mover_z=-4.2 (script moved it), respawn event seen;
        fixed camera pitch+yaw sign bug (was looking at sky). Pending: human opens window + clicks/plays.
     -> 2026-10-07: per-face near-plane culling (close boxes draw edges, no vanish-all);
        20 tests green (13 core, 4 server, 3 app); soak stable; screenshot re-verified.
     -> 2026-10-07: LIVE window photographed open on the Mac (docs-live-studio.png):
        title HUD "STUDIO /tmp/v3.rplace | asset=block | parts=5 | Tab=play ...", world renders
        (spawn/player/mover/kill/tower/grid). Keystroke injection blocked by OS (1002),
        so Tab/play-toggle needs a human hand. Window left running for the user.
     -> 2026-10-07: USER BUILT IN THE STUDIO unprompted: /tmp/v3.rplace grew 5->27 parts
        (22 gray blocks placed + S saved). Solved phantom-count + frozen-player mysteries.
        Fixed Play->Studio toggle persisting live mover pose (restore_mover_bases + test);
        repaired drifted mover in the user file via studio move. 21 tests green.
        Play-path screenshot of user level visually confirmed (docs-user-level.png).
     -> 2026-10-07: GUI --join path now covered: NetState moved to clone_player as
        PumpClient (shared, no dup) + ephemeral-port handshake/snapshot test; 22 green.
     -> 2026-10-07: release profile verified (docs hosting path): cargo build --release OK;
        release studio lists 27-part user level; release server + 2 release clients
        replicate 90 snapshots each (ids 1+2). Only human in-window play remains.
     -> 2026-10-07: live-window pixel audit: tower/spawn/mover/blocks/player colors all
        correct (no render bug); suite 21 green; 2-player re-verified on the 27-part user
        level (90 snapshots each, ids 1+2); added README hosting section.
