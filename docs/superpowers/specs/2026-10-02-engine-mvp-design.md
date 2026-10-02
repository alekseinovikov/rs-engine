# rs-engine MVP — Design Spec

- **Date:** 2026-10-02
- **Status:** Approved on 2026-10-02.
- **Roadmap and status:** [ROADMAP.md](../../../ROADMAP.md)

This is a living document: when a milestone settles a detail or changes a decision, update the
relevant section and add a row to the decision log (§15).

## 1. Context and goals

rs-engine is a learning project. The goal is to understand how a game engine works by building a
small 2D engine in Rust from scratch, on top of low-level crates for windowing, GPU access, math,
image decoding, audio mixing and font rasterization. Everything that makes an engine an engine is
written in this repository: the game loop, input handling, sprite rendering and batching, the
camera, assets, tilemaps, collision and platformer physics, animation, text, scenes, and an
entity-component system.

The MVP is complete when a small platformer (`games/platformer`) is fully playable on the engine
(§9).

Who does what:

- Claude writes the code and explains every decision.
- The user is new to game engines and has intermediate Rust experience (comfortable with traits,
  `Result` and ownership; has not built a large architecture yet). The user studies the code, asks
  questions and experiments.

Readability for a learner is therefore a first-class requirement (§12).

### Non-goals for the MVP

3D; editor tooling; scripting languages; networking; Windows and Web verification (the code stays
portable but is only checked on macOS); asset hot reloading; gamepads; generic ECS queries and
parallel scheduling; rigid-body physics. These live in the backlog in ROADMAP.md.

## 2. Key decisions

| Topic | Decision | Alternatives considered |
|---|---|---|
| Purpose | Learning: write the engine-level parts ourselves | Ship a game fast on ready-made crates; a public engine for others |
| Dimension | 2D | 3D; 2D designed up front for a later 3D extension |
| Architecture | Framework (in the spirit of raylib, LÖVE, macroquad) **plus our own mini-ECS** for game objects | Framework without an ECS (ECS after the MVP); Bevy-style app built on `bevy_ecs` |
| GPU API | wgpu (runs on Metal on macOS) | OpenGL (deprecated on macOS); raw Metal (locks us to one platform) |
| MVP criterion | Mini-platformer | Breakout; Asteroids; top-down shooter |
| Working mode | Claude writes the code, the user studies it | Mixed; the user writes with Claude as a mentor |
| Language | Everything in the repository in English; conversation in Russian | Russian docs and comments |

## 3. Architecture

A Cargo workspace with three crates. Dependencies point downward only:

```
games/platformer   binary: game rules, levels, screens
       │
       ▼
crates/engine      framework: loop, input, rendering, assets, audio, text, tilemaps,
       │           physics, animation, scenes + standard ECS components and systems
       ▼
crates/ecs         mini-ECS with zero dependencies: entities, storages, World,
                   resources, commands, events
```

**Boundary rule.** Code that would be useful in any other 2D game belongs in `engine`. Rules
specific to the platformer stay in the game: player controller tuning, enemies, coins, the level
file format, screens.

**Framework API.** The engine owns the window and the loop and calls the game back. The game
implements a trait:

```rust
pub trait Game {
    /// Advances the simulation by exactly one fixed step (`ctx.time.fixed_dt()`).
    fn update(&mut self, ctx: &mut Context);
    /// Submits draw commands for the current frame.
    fn draw(&mut self, ctx: &mut Context);
}

fn main() -> anyhow::Result<()> {
    // The closure runs once the window and the GPU exist, so the game can load assets.
    engine::run(Config::default(), |ctx| Platformer::new(ctx))
}
```

`Context` exposes the subsystems as public fields (`input`, `time`, `gfx`, `assets`, `audio`) so
the borrow checker can split borrows, for example reading `ctx.input` while drawing through
`ctx.gfx`. Exact signatures are settled in the milestone plans; `Context` grows milestone by
milestone.

**ECS integration.** The ECS is a tool the game uses to store its objects. The core subsystems
(loop, renderer, input, audio, assets) do not use it; ECS-facing components and systems sit on top
of them. Systems are plain functions with explicit parameters, called in order from `update`. The
call order *is* the schedule:

```rust
fn update(&mut self, ctx: &mut Context) {
    let dt = ctx.time.fixed_dt();
    player_input(&self.world, &ctx.input);
    engine::physics::move_bodies(&self.world, &self.level.tilemap, dt);
    engine::physics::detect_overlaps(&self.world);
    collect_coins(&self.world, &mut ctx.audio);
    engine::animation::advance(&self.world, dt);
    self.world.apply_commands();
    self.world.clear_events();
}
```

`engine` provides standard components (`Transform`, `Sprite`, `Body`, `Collider`, `Animator`) and
the systems that operate on them. The game adds its own components (`Player`, `Coin`, `Enemy`, …)
and systems.

## 4. Frame lifecycle

A fixed timestep with an accumulator, as described in Glenn Fiedler's "Fix Your Timestep!":

```
every frame (paced by vsync):
    accumulator += real frame time, clamped to 0.25 s    // avoids the "spiral of death"
    while accumulator >= fixed_dt (1/60 s by default):
        game.update(ctx)          // logic and physics always step exactly fixed_dt
        input.end_tick()          // "just pressed/released" flags live for one tick
        accumulator -= fixed_dt
    gfx.begin_frame()
    game.draw(ctx)                // the game submits sprites, tiles, text
    gfx.end_frame()               // sort, batch, upload, render passes, present
```

- Fixed steps make physics deterministic and independent of the frame rate.
- Edge-triggered input ("just pressed") is latched per fixed tick, not per frame. A press that
  arrives during a frame with zero ticks survives until the next tick; a frame with two ticks does
  not see it twice.
- Rendering draws the latest simulation state without interpolation. Interpolation is in the
  backlog.

## 5. Engine subsystems

**Rendering (wgpu).**

- Coordinate convention: world units are pixels of the virtual resolution; +x points right and
  +y points down, matching screen, texture and tilemap row order.
- Immediate-style API: `gfx.draw_sprite(...)`, `gfx.draw_rect(...)` and `gfx.draw_text(...)`
  collect instances for the current frame. Nothing is retained between frames.
- One sprite pipeline for everything: a unit quad plus per-instance data (position, size,
  rotation, UV rectangle, color). Solid rectangles use a 1×1 white texture region; text glyphs are
  sprites too.
- Ordering: a stable sort by layer, then by submission order. Consecutive instances that share a
  texture form one draw call (a batch).
- `Camera2D` (position, zoom) produces a view-projection matrix stored in a uniform buffer. A
  separate screen-space mode is used for UI.
- Pixel-perfect output: the scene renders into an offscreen texture at a virtual resolution
  (default 480×270). A second pass upscales it to the window by the largest integer factor that
  fits and letterboxes the rest. Nearest-neighbour sampling everywhere.
- Surface errors: `Lost` or `Outdated` → reconfigure the surface and skip the frame; `Timeout` →
  skip the frame; out of memory → exit with an error.

**Assets.** Load by path and get back a typed, lightweight `Handle<T>`: an index that is `Copy`
and needs no lifetimes or `Rc`. Assets are cached by path and loaded synchronously when a scene
starts. The asset root is the workspace `assets/` directory during development and `assets/` next
to the executable in release builds. GPU textures are owned by the renderer; handles refer to them.

**Input.** Keyboard: `is_down`, `just_pressed`, `just_released`, latched per tick (§4). Minimal
mouse state. A small action map (for example `Jump` → Space, W or Up) for games.

**Time.** Fixed delta, real frame delta, elapsed time, FPS.

**Tilemap.** A grid of tile ids with a tile size. Renders only the tiles visible to the camera.
Collision queries: is the tile at a point solid, which tiles overlap a rectangle. The level *file
format* is game-specific (§9).

**Physics.** Kinematic, not dynamic.

- `Aabb` and overlap tests.
- `Body` (velocity, contact flags) and `Collider` components.
- `move_bodies` moves each body along X and resolves against solid tiles, then along Y and
  resolves again, and sets `on_ground`, `hit_wall` and `hit_ceiling`. Gravity comes from a
  resource.
- Velocities are clamped so that no body moves more than one tile per tick, which rules out
  tunneling.
- `detect_overlaps` emits `Overlap { a, b }` events for intersecting colliders. It is a naive
  O(n²) check, which is fine for dozens of entities.

**Animation.** `AnimationClip` (atlas frames, frame duration, looping) and an `Animator`
component. A system advances time and updates the region of the `Sprite`. Sprites can flip
horizontally.

**Camera follow.** Frame-rate-independent exponential smoothing, a dead zone, clamping to the
level bounds, pixel snapping.

**Text.** fontdue rasterizes a CC0 pixel font into a glyph atlas at load time. Text renders
through the sprite batcher.

**Audio.** A thin wrapper over kira: sound effects as static sounds, looping music as streamed
sounds, volume control. If no audio device is available, log a warning and continue silently.

**Scenes.** A `Scene` trait: `update` returns a transition (none, push, pop, replace, quit), and
`draw` renders. `SceneStack` implements `Game`, so a game can hand the engine a stack of scenes.
Each scene owns its data, including its own ECS `World`. Fade transitions between scenes.

**Debug overlay.** Toggled by the game (F1): collider outlines, contact flags, FPS.

## 6. Mini-ECS (`crates/ecs`)

- **Entity:** `{ index: u32, generation: u32 }`, `Copy`. The allocator keeps a generation per
  slot and a free list. A despawned entity's slot is reused with a higher generation, so stale
  handles are detected: `is_alive` returns false and lookups return `None`.
- **Storage:** one `SparseSet<T>` per component type, as in EnTT. A sparse array maps an entity
  index to a dense index; dense arrays hold entities and components side by side. Insert, remove
  and lookup are O(1) (removal uses swap-remove); iteration walks a packed array.
- **World:** component storages live in a `HashMap<TypeId, …>`, each behind a `RefCell`.
  `world.read::<T>()` and `world.write::<T>()` return guards, so a system can hold
  `write::<Transform>()` and `read::<Collider>()` at the same time. Conflicting borrows panic with
  a clear message: runtime borrow checking, a deliberate learning point. Storages are registered
  explicitly or on first insert; reading an unregistered type panics with a hint.
- **Queries are explicit joins:** iterate one storage and look the others up by entity.

  ```rust
  let mut bodies = world.write::<Body>();
  let mut transforms = world.write::<Transform>();
  for (entity, body) in bodies.iter_mut() {
      let Some(transform) = transforms.get_mut(entity) else { continue };
      transform.position += body.velocity * dt;
  }
  ```

- **Resources:** singletons by type (`resource::<T>()`, `resource_mut::<T>()`), for example
  gravity or the score.
- **Commands:** structural changes (spawn, despawn, insert, remove) are queued while systems hold
  storage borrows, then applied with `world.apply_commands()` between systems.
- **Events:** `world.send(event)` and `world.events::<T>()`. Events are cleared with
  `world.clear_events()` once per tick, so writers must run before readers.
- No `unsafe`, no macros. Generic queries, a scheduler and parallelism come after the MVP. They
  are exactly where real ECS implementations need `unsafe` or heavy type machinery, which makes a
  good topic for a learning note.

## 7. Error handling and logging

- Engine: fallible public APIs return `Result<T, engine::Error>` (thiserror). This covers window
  creation, GPU initialization (no adapter, device request failure), asset I/O and decoding (the
  error includes the path) and audio initialization.
- Game: `anyhow::Result` with context in `main`.
- Panics only for programmer errors (ECS borrow conflicts, unregistered components, invalid
  handles), always with a message that says how to fix the problem.
- Assets load when a scene starts. A missing or broken asset fails fast and names its path; it
  never fails mid-game.
- Logging: the `log` facade with `env_logger` (`RUST_LOG`, default `info`). wgpu validation errors
  show up in the log in debug builds.

## 8. Testing and CI

- Unit tests for all pure logic: the ECS (allocation, generations, sparse-set operations, borrow
  conflicts, commands, events), the timestep accumulator and clamping, input transitions, camera
  and projection math, atlas UVs, integer-scale and letterbox math, CPU-side batch building, level
  parsing and autotiling, collision resolution (landing, walls, ceilings, corners, fast movement),
  animation timing, scene transitions.
- Test-first for pure logic.
- No GPU in automated tests. Every milestone ends with a runnable example or game state that the
  user checks by hand against the milestone's "done when" list.
- CI on GitHub Actions, active once a remote exists: `cargo fmt --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, a build of
  all targets. Ubuntu and macOS runners.

## 9. Demo game: platformer

- Three short levels in an ASCII format, one character per tile: `#` ground, `^` spikes, `o` coin,
  `E` enemy, `P` player start, `F` goal flag, `.` empty. The files are editable in any text editor,
  and parse errors report the line and column. Ground tiles are autotiled from their neighbours.
- Player: running with acceleration and deceleration, a variable-height jump, coyote time, jump
  buffering. Idle, run, jump and fall animations. Faces the movement direction.
- Coins (with a counter), spikes (death), patrolling enemies, a goal flag (next level). Enemies
  turn around at walls and ledges. Stomping from above defeats them and bounces the player; side
  contact kills the player.
- Three lives. A death respawns the player at the level start; zero lives means Game Over. After
  the last level comes a Victory screen with the total number of coins.
- Screens: Title, Pause (Esc), Game Over, Victory; fade transitions.
- HUD: coins, lives, level number. Sounds: jump, coin, stomp, hurt, win; looping background music.
- Debug overlay on F1.
- Art, font and sounds: CC0 packs (for example Kenney's Pixel Platformer); generated placeholder
  shapes until they are added. Every download is confirmed with the user first, and sources and
  licenses are recorded in `assets/CREDITS.md`.

## 10. Tech stack (versions as of 2026-10-02)

| Crate | Version | Purpose |
|---|---|---|
| winit | 0.30 | Window and OS events |
| wgpu | 30 | GPU access (Metal on macOS) |
| glam | 0.33 | Vectors and matrices |
| image | 0.25 | PNG decoding (only the `png` feature) |
| kira | 0.12 | Audio playback and mixing |
| fontdue | 0.9 | Font rasterization |
| bytemuck | 1 | Safe casting of vertex data to bytes |
| pollster | 1 | Blocking on wgpu's async initialization |
| log / env_logger | 0.4 / 0.11 | Logging |
| thiserror / anyhow | 2 / 1 | Errors in the engine / in the game |

Toolchain: Rust stable, edition 2024, resolver 3. Versions live in `[workspace.dependencies]`.
Each milestone adds only the crates it needs.

## 11. Repository layout

```
rs-engine/
├── CLAUDE.md              guidance for Claude sessions
├── ROADMAP.md             milestones and status
├── Cargo.toml             workspace: shared dependency versions and lints
├── rust-toolchain.toml    stable + rustfmt + clippy
├── crates/ecs/
├── crates/engine/         + examples/ (at least one per milestone)
├── games/platformer/
├── assets/                shared assets + CREDITS.md
├── docs/learning/         one learning note per milestone
├── docs/superpowers/      specs and milestone implementation plans
└── .github/workflows/     CI
```

## 12. Code style for a learning codebase

- Every module starts with `//!` docs that explain the concept behind it and link to further
  reading.
- Comments explain *why*; the code shows *what*.
- Explicit, simple code over clever abstractions: few generics, no macro magic, no `unsafe`.
- `missing_docs` is a warning in `engine` and `ecs`; CI treats warnings as errors.
- Small focused files. A file growing past a few hundred lines is a signal to split it.

## 13. Learning materials

- `docs/learning/NN-<name>.md` for every milestone: what we built, the key concepts, where they
  live in the code, pitfalls, experiments to try ("change X and observe Y"), further reading.
- A git tag at the end of every milestone (`m0-foundation`, `m1-game-loop`, …), so any stage can
  be revisited with `git checkout <tag>`.
- Examples: `cargo run -p engine --example <name>`.

## 14. Open questions

Each one is resolved in the milestone named.

- License (suggested: MIT OR Apache-2.0) and whether to create a GitHub remote: M0.
- The exact art pack and the virtual resolution (default 480×270): M4.
- Font and sound packs: M10.

## 15. Decision log

| Date | Decision |
|---|---|
| 2026-10-02 | Learning-focused 2D engine; Claude writes the code, the user studies it. |
| 2026-10-02 | Framework plus our own mini-ECS; a three-crate workspace (`ecs` → `engine` → `platformer`). |
| 2026-10-02 | MVP criterion: a mini-platformer. Everything in the repository is written in English. |
