# Roadmap to the MVP

The MVP is reached when the mini-platformer is playable from the title screen to the victory
screen on our own engine. Design and decision log:
[docs/superpowers/specs/2026-10-02-engine-mvp-design.md](docs/superpowers/specs/2026-10-02-engine-mvp-design.md).

**Current milestone:** M2 — Hello GPU (not started)

Legend: ⬜ not started · 🟨 in progress · ✅ done

| # | Milestone | What you see at the end | Status |
|---|---|---|---|
| M0 | Foundation | `cargo run -p platformer` prints a greeting from the engine; all checks pass | ✅ |
| M1 | Window, game loop, input | A window with FPS in its title; key presses in the log | ✅ |
| M2 | Hello GPU | A cleared window and a colored triangle | ⬜ |
| M3 | Sprite batching & camera | 10 000 moving rectangles; a panning, zooming camera | ⬜ |
| M4 | Textures & pixel-perfect rendering | Crisp pixel-art sprites from an atlas at any window size | ⬜ |
| M5 | Mini-ECS | Hundreds of bouncing sprite entities, spawned and despawned | ⬜ |
| M6 | Assets & tilemaps | Level 1 loaded from a text file; a scrolling camera | ⬜ |
| M7 | Platformer physics | A player who runs and jumps through the level | ⬜ |
| M8 | Animation & camera follow | An animated character; a smooth following camera | ⬜ |
| M9 | Gameplay objects | Coins, spikes, enemies, a goal flag; level 1 can be beaten | ⬜ |
| M10 | Text, HUD & audio | A HUD on screen; sound effects and music | ⬜ |
| M11 | Scenes & game flow | Title, pause, game over, three levels, victory | ⬜ |
| M12 | Polish & v0.1.0 | The finished MVP, tagged `v0.1.0` | ⬜ |

## How we work through a milestone

One milestone at a time: the next one starts only when the current one is done and the user
agrees. Each milestone is developed on its own branch, `milestone/mN-<name>`, and reaches `main`
through a pull request.

1. Clarify open details with the user (a short brainstorm).
2. Write the implementation plan: `docs/superpowers/plans/YYYY-MM-DD-mN-<name>.md`.
3. Implement in small steps on the milestone branch, test-first for pure logic, committing after
   each step.
4. Verify: fmt, clippy, tests, run the example. Open the pull request; CI must be green.
5. Write the learning note: `docs/learning/NN-<name>.md`.
6. Mark the milestone done here and update "Current milestone".
7. The user runs the example, reads the code and reviews the pull request.
8. After the user's OK: merge the pull request with a merge commit and tag the merge commit
   (`mN-<name>`).

---

## M0 — Foundation

**Tag:** `m0-foundation` · **Goal:** a working toolchain and an empty but healthy workspace.

Scope:

- Install Rust with rustup (stable, with rustfmt and clippy); `rust-toolchain.toml`.
- Cargo workspace (edition 2024, resolver 3): `crates/ecs` (lib), `crates/engine` (lib),
  `games/platformer` (bin); `[workspace.dependencies]` and `[workspace.lints]`.
- `README.md` and the MIT OR Apache-2.0 dual license.
- GitHub Actions CI: fmt, clippy, tests, build (Ubuntu and macOS) in the public repository
  `alekseinovikov/rs-engine`.
- Learning note `00-foundation.md`.

Done when:

- `cargo run -p platformer` prints a greeting that comes from the `engine` crate.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings` and
  `cargo test --workspace` pass.

Learn: Cargo workspaces, crates vs modules, editions, toolchains, rustfmt and clippy, CI basics.

## M1 — Window, game loop, input

**Tag:** `m1-game-loop` · **Goal:** a window driven by a proper game loop.

Scope:

- winit `ApplicationHandler`; window creation; quit on the close button or Esc.
- The `Game` trait, `Context`, `engine::run`.
- `Time`: a fixed timestep (60 Hz) with an accumulator and a frame-time clamp; an FPS counter in
  the window title.
- `Input`: keyboard down / just pressed / just released, latched per fixed tick; minimal mouse
  state.
- Logging (`log` + `env_logger`).
- Example `window`.

Done when:

- The example opens a window, shows FPS in the title and logs key presses.
- Unit tests cover the accumulator (ticks per frame, clamping) and input transitions.

Learn: event loops and OS events, fixed vs variable timestep ("Fix Your Timestep!"), input edge
detection, separating update from render.

## M2 — Hello GPU

**Tag:** `m2-hello-gpu` · **Goal:** the first pixels from the GPU.

Scope:

- wgpu setup: instance, surface, adapter, device, queue; surface configuration (sRGB, vsync);
  reconfiguration on resize; handling of lost and outdated surfaces.
- A render pass that clears the screen.
- The first render pipeline: a WGSL shader, a vertex buffer, a colored triangle; then a quad with
  an index buffer.
- Example `triangle`.

Done when:

- The triangle renders, resizing works, and the log shows no wgpu validation errors.

Learn: how a GPU frame works (encoder → render pass → submit → present), vertex and fragment
shaders, buffers, pipeline state, the swapchain, sRGB.

## M3 — Sprite batching & camera

**Tag:** `m3-sprite-batch` · **Goal:** thousands of rectangles drawn efficiently in world
coordinates.

Scope:

- `Camera2D` (position, zoom) → a view-projection uniform; world ↔ screen conversion; +y points
  down.
- An immediate-style API (`gfx.draw_rect`) that collects per-frame instances; instanced rendering
  of a unit quad.
- Ordering by layer, then by submission order; batching.
- Example `quads`: 10 000 moving rectangles; the arrow keys pan the camera, +/- zoom.

Done when:

- The example holds 60 FPS (vsync).
- Unit tests cover camera math and CPU-side batch building.

Learn: coordinate spaces (world, view, clip, screen), orthographic projection, uniforms and bind
groups, instancing, draw calls and why batching matters.

## M4 — Textures & pixel-perfect rendering

**Tag:** `m4-textures` · **Goal:** textured sprites from an atlas, crisp at any window size.

Scope:

- PNG → GPU texture (`image`); a nearest-neighbor sampler; alpha blending.
- A grid texture atlas with UV regions; `gfx.draw_sprite` with flip and tint; batches split by
  texture.
- An offscreen render target at the virtual resolution (default 480×270), upscaled by an integer
  factor with letterboxing.
- The first real art: a CC0 pack (for example Kenney's Pixel Platformer). Ask before downloading;
  until then use a generated texture.
- Example `sprites`.

Done when:

- Sprites stay crisp, without shimmering, at any window size.
- Unit tests cover atlas UVs and integer-scale and letterbox math.

Learn: textures, UVs, samplers and filtering, alpha blending, atlases and texture bleeding, render
targets and multi-pass rendering, virtual resolution.

## M5 — Mini-ECS

**Tag:** `m5-ecs` · **Goal:** our own ECS and the first engine components built on it.

Scope:

- `crates/ecs`: a generational `Entity` and its allocator; `SparseSet<T>` storage; a `World` with
  `RefCell` storages (`read` / `write`), spawn, despawn, insert, remove; resources; commands;
  events. No `unsafe`, no macros.
- `crates/engine`: `Transform` and `Sprite` components; a `draw_sprites` system.
- Example `bunnies`: keys spawn and despawn hundreds of bouncing sprites; the entity count is in
  the window title.

Done when:

- `ecs` has thorough unit tests: generations and stale handles, swap-remove, borrow conflicts,
  commands, events.
- The example runs smoothly.

Learn: what an ECS is and why (data-oriented design, composition over inheritance), generational
indices, sparse sets, `RefCell` as runtime borrow checking, deferred structural changes, how Bevy,
EnTT and Unity DOTS differ (archetypes, schedulers).

## M6 — Assets & tilemaps

**Tag:** `m6-tilemap` · **Goal:** levels loaded from files and rendered as tilemaps.

Scope:

- `Assets`: typed handles, a cache by path, asset root resolution (development vs release).
- `Tilemap`: grid storage, rendering of visible tiles only, solidity queries.
- Game: the ASCII level format and its parser (errors with line and column), autotiling of ground
  tiles, spawn points.
- Camera scrolling clamped to the level bounds.

Done when:

- The platformer shows level 1 and the camera scrolls over it with the arrow keys.
- Unit tests cover parsing, autotiling and tile queries.

Learn: handles vs references, caching, data-driven design, tilemaps and culling, autotiling.

## M7 — Platformer physics

**Tag:** `m7-physics` · **Goal:** a player who runs and jumps through the level with good game
feel.

Scope:

- Engine: `Aabb`; `Body` and `Collider` components; `move_bodies` (axis-separated movement against
  solid tiles, contact flags, a gravity resource, a velocity clamp).
- Game: the player controller with acceleration and deceleration, a variable-height jump, coyote
  time and jump buffering; an input action map.
- Debug overlay (F1): colliders and contact flags.

Done when:

- The player runs and jumps through level 1 without sticking to walls or falling through floors.
- Unit tests cover landing, wall and ceiling hits, corners and maximum-speed movement.

Learn: kinematic vs dynamic physics, AABB collision resolution, why platformers avoid realistic
physics, game feel, tunneling.

## M8 — Animation & camera follow

**Tag:** `m8-animation` · **Goal:** a living character and a smooth camera.

Scope:

- Engine: `AnimationClip`, `Animator`, the animation system; horizontal flip.
- Game: player state → clip (idle, run, jump, fall).
- Engine: camera follow with exponential smoothing, a dead zone, level bounds and pixel snapping.

Done when:

- The player animates according to its state, and the camera follows without jitter.
- Unit tests cover frame timing and looping, and camera clamping.

Learn: frame-based animation, animation state machines, frame-rate-independent smoothing, jitter
and pixel snapping.

## M9 — Gameplay objects

**Tag:** `m9-gameplay` · **Goal:** the platformer becomes a game.

Scope:

- Engine: `detect_overlaps` → `Overlap` events.
- Game: coins, spikes, patrolling enemies (turning at walls and ledges; a stomp vs a side hit), the
  goal flag, three lives and respawning.

Done when:

- Level 1 can be completed, and it can be lost.
- Unit tests cover the pure rules (stomp vs side hit, lives and respawning).

Learn: triggers vs solid collisions, events for decoupling systems, entity lifecycle, simple AI
states.

## M10 — Text, HUD & audio

**Tag:** `m10-text-audio` · **Goal:** text on screen and sound.

Scope:

- Engine: font → glyph atlas (fontdue); `gfx.draw_text`; screen-space drawing for UI.
- Game: a HUD (coins, lives, level); FPS in the debug overlay.
- Engine: an audio wrapper over kira with sound effects, looping music and volume; a silent
  fallback without an audio device. CI: the ALSA dev package on Linux.
- Game: sound effects (jump, coin, stomp, hurt, win) and music. CC0 only; ask before downloading.

Done when:

- The HUD shows correct values, sounds play on game events, and the game still runs when audio is
  unavailable.

Learn: font rasterization and glyph metrics, glyph atlases, world vs screen space, audio mixing,
static vs streamed sounds.

## M11 — Scenes & game flow

**Tag:** `m11-scenes` · **Goal:** a complete game from the title screen to the ending.

Scope:

- Engine: a `Scene` trait and a `SceneStack` (push, pop, replace, quit) that implements `Game`;
  fade transitions.
- Game: Title → Playing → Pause → Game Over or the next level → Victory with the total coins;
  three levels; restart.

Done when:

- The whole flow works from the title screen to victory and back.
- Unit tests cover scene stack transitions.

Learn: game state management, scene stacks, per-scene ownership of data (each scene owns its
`World`).

## M12 — Polish & v0.1.0

**Tag:** `v0.1.0` · **Goal:** ship the MVP.

Scope:

- Tune the game feel and the levels; fix bugs found in playtesting.
- Release build: a stable 60 FPS; asset paths work in release.
- README: a screenshot or GIF, controls, build instructions, an architecture overview, credits.
- A retrospective learning note: what we built, what we would change, what comes next.

Done when: the MVP definition of done below holds.

---

## MVP definition of done

- The platformer is playable from the title screen to victory: three levels, coins, spikes,
  enemies, lives, pause, game over, victory.
- A stable 60 FPS in a release build on the development machine.
- Everything the game uses, except its own rules and content, lives in `engine` or `ecs`.
- fmt, clippy (`-D warnings`) and all tests pass; CI is green if a remote exists.
- Every milestone has a learning note and a git tag.

## Backlog (after the MVP)

- ECS: generic queries, a system scheduler, parallel execution, archetype storage (compared with
  sparse sets).
- Asset hot reloading; an egui debug UI; gamepad support (gilrs).
- Tiled or LDtk level import; one-way and moving platforms; particles.
- Render interpolation; a transform hierarchy; spatial partitioning for collisions.
- A save system; Windows and Web (WASM) builds; post-processing and lighting shaders; 3D.
