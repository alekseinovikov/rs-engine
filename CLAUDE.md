# CLAUDE.md

Guidance for Claude Code sessions in this repository.

## Project

rs-engine is a **learning project**: a small 2D game engine in Rust, built from scratch on top of
low-level crates to understand how game engines work. The MVP is done when the mini-platformer in
`games/platformer` is playable from the title screen to victory.

- Plan and status: [ROADMAP.md](ROADMAP.md)
- Design and decision log: [docs/superpowers/specs/2026-10-02-engine-mvp-design.md](docs/superpowers/specs/2026-10-02-engine-mvp-design.md)

## Working mode

- The user is new to game engines and has intermediate Rust (traits, `Result`, ownership; no large
  architectures yet).
- **Claude writes the code; the user studies it**, asks questions and experiments. Explain every
  non-obvious decision in chat as you make it.
- **Language:** everything in the repository (code, comments, docs, commit messages) is in
  English. Talk to the user in Russian.

## Session start

1. Read ROADMAP.md to find the current milestone and its status.
2. Check the branch: milestone work happens on `milestone/mN-<name>`. If an unmerged milestone
   branch exists, continue there.
3. If a plan for the milestone exists in `docs/superpowers/plans/`, continue from the first
   unchecked step.
4. Skim `git log --oneline -15` and the latest note in `docs/learning/`.
5. Read the design spec before changing the architecture.

## Milestone workflow

One milestone at a time: start the next one only when the current one is done and the user agrees.
Each milestone is developed on its own branch, `milestone/mN-<name>`, and reaches `main` through a
pull request.

1. Clarify open details with the user.
2. Create the branch from an up-to-date `main`.
3. Write the plan: `docs/superpowers/plans/YYYY-MM-DD-mN-<name>.md`; tick its checkboxes as steps
   are completed.
4. Implement in small steps, test-first for pure logic, committing after each step.
5. Verify: fmt, clippy, tests, run the milestone example. Push the branch and open a pull request;
   CI must be green.
6. Write the learning note `docs/learning/NN-<name>.md`: concepts, where they live in the code,
   pitfalls, experiments to try, further reading.
7. Update ROADMAP.md (status, current milestone) and add significant decisions to the spec's
   decision log.
8. The user runs the example, reads the code and reviews the pull request.
9. After the user's OK: merge the pull request with a merge commit, tag the merge commit on `main`
   (`mN-<name>`) and push the tag.

## Architecture

A Cargo workspace; dependencies point downward only:

- `games/platformer`: the MVP game (binary): player controller, enemies, coins, level format,
  screens.
- `crates/engine`: the framework: loop, input, rendering, assets, audio, text, tilemaps, physics,
  animation, scenes, plus standard ECS components and systems.
- `crates/ecs`: a mini-ECS with zero dependencies: generational entities, sparse-set storages,
  `World`, resources, commands, events.

Rules:

- Code useful to any 2D game goes in `engine`; platformer-specific rules stay in the game.
- The engine is a framework: the game implements `Game` (`update` at a fixed timestep, `draw` once
  per frame) and gets a `Context` with public subsystem fields (`input`, `time`, `gfx`, `assets`,
  `audio`).
- ECS systems are plain functions with explicit parameters, called in order from `update`; the
  call order is the schedule.
- The core subsystems (loop, renderer, input, audio, assets) do not depend on the ECS.
- World units are virtual-resolution pixels; +y points down.

## Code style: this codebase is a textbook

- Every module starts with `//!` docs that explain its concept; comments explain *why*.
- Explicit and simple over clever: few generics, no macro magic, **no `unsafe`**.
- Public items are documented (`missing_docs` warns in `engine` and `ecs`).
- Errors: `engine::Error` (thiserror) in the engine, `anyhow` in the game. Panic only on
  programmer errors, with a message that says how to fix the problem.
- Unit tests for all pure logic; no GPU in tests.
- Small, focused files.

## Dependencies

winit 0.30, wgpu 30, glam 0.33, image 0.25, kira 0.12, fontdue 0.9, plus bytemuck, pollster, log,
env_logger, thiserror and anyhow. Versions live in `[workspace.dependencies]`; add a crate only in
the milestone that needs it. Check the current API in the docs (context7 or docs.rs) before writing
code against it: wgpu changes between major versions.

## Assets

CC0 or similarly permissive only; record the source and license in `assets/CREDITS.md`. Ask the
user before downloading anything.

## Commands (available after M0)

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
cargo run -p platformer
cargo run -p engine --example <name>
```

## Out of scope for the MVP

3D, an editor, scripting, networking, hot reloading, gamepads, Windows/Web verification, generic
ECS queries and schedulers: see the backlog in ROADMAP.md. Do not add these unless the user asks.
