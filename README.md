# rs-engine

A small 2D game engine written in Rust from scratch, to learn how game engines work. The MVP goal
is a mini-platformer that is playable on this engine from the title screen to victory.

**Status:** early development. See [ROADMAP.md](ROADMAP.md) for the milestones and progress.

## Workspace

| Crate | Path | What it is |
|---|---|---|
| `ecs` | [`crates/ecs`](crates/ecs) | A minimal entity-component system |
| `engine` | [`crates/engine`](crates/engine) | The engine framework |
| `platformer` | [`games/platformer`](games/platformer) | The MVP demo game |

## Getting started

Install Rust with [rustup](https://rustup.rs). The repository pins its toolchain in
`rust-toolchain.toml`, so rustup installs the right version and components automatically.

```bash
cargo run -p platformer                  # run the game
cargo test --workspace                   # run all tests
cargo doc --workspace --no-deps --open   # browse the API docs
```

## Documentation

- [ROADMAP.md](ROADMAP.md): milestones and status
- [Design spec](docs/superpowers/specs/2026-10-02-engine-mvp-design.md): architecture and decisions
- [Learning notes](docs/learning): what each milestone teaches

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
this project by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.
