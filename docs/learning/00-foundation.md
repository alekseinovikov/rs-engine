# 00 — Foundation

**Milestone:** M0 · **Tag:** `m0-foundation`

In this milestone we installed the Rust toolchain and gave the repository its final shape: a Cargo
workspace with three crates, shared lints, tests, CI and a license. There is almost no engine code
yet, but every later milestone builds on these decisions.

## What we built

```
rs-engine/
├── Cargo.toml            the workspace manifest
├── Cargo.lock            exact versions of every dependency (committed)
├── rust-toolchain.toml   which Rust toolchain this repository uses
├── crates/ecs/           library: the future entity-component system (M5)
├── crates/engine/        library: the engine framework
├── games/platformer/     binary: the game; for now it prints the engine's greeting
└── .github/workflows/    CI
```

`cargo run -p platformer` prints `rs-engine 0.1.0 (ecs 0.1.0)`. The binary calls
`engine::greeting()`, which reads `ecs::VERSION`, so this one line crosses both crate boundaries.

## Key concepts

### rustup, rustc and cargo

- **rustc** is the compiler. You rarely call it directly.
- **cargo** is the build tool and package manager. It resolves dependencies, calls rustc, runs
  tests, and hosts plugins such as `cargo fmt` and `cargo clippy`.
- **rustup** installs and switches *toolchains* (stable, beta, nightly or an exact version) and
  their *components* (rustfmt, clippy, rust-docs, …). It puts small proxy programs named `cargo`
  and `rustc` into `~/.cargo/bin`; each proxy picks the right toolchain for the current directory.
- **`rust-toolchain.toml`** tells those proxies which toolchain to use in this repository. Ours
  asks for the `stable` channel plus `rustfmt` and `clippy`, so a fresh clone or a CI runner gets
  everything it needs automatically.

### Crates, packages and workspaces

- A **crate** is one compilation unit: a library (`src/lib.rs`) or a binary (`src/main.rs`). A
  **package** is a directory with a `Cargo.toml` that builds one or more crates.
- A **workspace** groups packages that are developed together. They share one `Cargo.lock` and one
  `target/` directory, so each dependency is compiled once for all of them.
- Our root `Cargo.toml` is a **virtual manifest**: it has a `[workspace]` section but no
  `[package]`.
- **Inheritance:** `[workspace.package]` holds shared metadata (version, edition, license, …) and
  `[workspace.dependencies]` holds shared dependency declarations. Members opt in with
  `version.workspace = true`, `ecs.workspace = true` and so on, so every version is written in
  exactly one place.
- **Path dependencies:** `ecs = { path = "crates/ecs" }` points at a crate in this repository
  instead of one published on crates.io.

### Why three crates instead of three modules

Modules (`mod render;`) organize code *inside* a crate. Crates are separate compilation units with
explicit dependencies, which turns our architecture rules into compiler errors:

- `ecs` cannot accidentally use graphics code: it has no dependencies at all.
- `engine` cannot depend on the game: Cargo forbids dependency cycles.
- `ecs` can be built, tested and understood on its own.

### Editions and the resolver

An **edition** (2015, 2018, 2021, 2024) is an opt-in set of language changes that would otherwise
break existing code. Crates of different editions link together without problems, so the ecosystem
never splits. We use **2024**, the latest. `resolver = "3"` selects Cargo's newest dependency
resolver; its main difference from version 2 is that it takes into account the minimum Rust
version that dependencies declare.

### Lints, rustfmt and clippy

- **Lints** are compiler checks whose level you can tune: allow, warn, deny or forbid. Ours live in
  `[workspace.lints]`, and every crate opts in with `[lints] workspace = true`:
  - `unsafe_code = "deny"`: our code never uses `unsafe`. It is `deny` rather than `forbid` so
    that one narrowly scoped `#[allow]` stays possible if a dependency's derive macro ever
    generates an `unsafe impl` for us.
  - `missing_docs = "warn"`: every public item needs a doc comment.
- **rustfmt** (`cargo fmt`) formats all code in one standard style, so diffs only show real
  changes.
- **clippy** (`cargo clippy`) adds hundreds of extra lints that catch bugs and unidiomatic code.
- CI runs clippy with `-D warnings`, which turns every warning into an error, so warnings never
  pile up.

### Three kinds of tests

M0 already has all three:

- **Unit tests** live next to the code in a `#[cfg(test)] mod tests` block (see
  `crates/engine/src/lib.rs`). `#[cfg(test)]` compiles the module only for `cargo test`. Unit tests
  can reach private items.
- **Doc tests** are the code examples in `///` comments. `cargo test` compiles and runs them, so
  the examples in the documentation can never go stale. See the example on `engine::greeting`.
- **Integration tests** live in a package's `tests/` directory and use the package from the
  outside, the way a user would. `games/platformer/tests/startup.rs` runs the compiled game: Cargo
  hands the test the binary's path in the `CARGO_BIN_EXE_platformer` environment variable. This
  test is temporary. Once the game opens a window (M6), running the binary would block until the
  window is closed, and tests of the game's logic will replace it.

### Continuous integration

`.github/workflows/ci.yml` runs on every push to `main` and on every pull request, on Ubuntu and on
macOS. It installs the toolchain from `rust-toolchain.toml`, restores the build cache, then runs
`cargo fmt --check`, `cargo clippy -D warnings` and `cargo test`. Two operating systems catch
platform-specific breakage early: winit, wgpu and kira use different backends on each. `cargo test`
also compiles every example, so a broken example fails CI too.

### Licensing

Most of the Rust ecosystem, including Rust itself, uses the dual license **MIT OR Apache-2.0**:
users may pick either one. MIT is short and permissive; Apache-2.0 adds an explicit patent grant.
Sharing the ecosystem's license makes it easy to move code between projects.

## Where to look in the code

- `Cargo.toml`: workspace members, shared metadata, shared dependencies and lints.
- `crates/engine/Cargo.toml`: how a member inherits all of that.
- `crates/engine/src/lib.rs`: crate docs, a doc test and a unit test.
- `games/platformer/tests/startup.rs`: an integration test that runs a binary.

## Experiments to try

1. Change the wording of `engine::greeting()` and run `cargo test --workspace`. Which tests fail,
   and why? (The unit test checks the format; the integration test only checks that the binary
   prints whatever `greeting()` returns.)
2. Set `version = "0.2.0"` in `[workspace.package]` and run the game: both crates report the new
   version because both inherit it.
3. Add `unsafe {}` to any function and run `cargo build`. Then delete the `///` comment above
   `engine::VERSION` and run `cargo clippy --workspace -- -D warnings`.
4. Add `engine.workspace = true` under a new `[dependencies]` section in `crates/ecs/Cargo.toml`
   and run `cargo build`: Cargo reports a dependency cycle. Revert the change.
5. Break the formatting of a file, run `cargo fmt --all --check`, then fix it with
   `cargo fmt --all`.
6. Run `cargo tree` to see the dependency graph, and `cargo doc --workspace --no-deps --open` to
   browse our documentation as a website.

## Further reading

- The Cargo Book: [Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html) and
  [the `[lints]` section](https://doc.rust-lang.org/cargo/reference/manifest.html#the-lints-section)
- [The Rust Edition Guide](https://doc.rust-lang.org/edition-guide/)
- The rustup book: [Overrides](https://rust-lang.github.io/rustup/overrides.html), which explains
  how `rust-toolchain.toml` works
- The Rust Book, [chapter 11: Writing Automated Tests](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [The list of clippy lints](https://rust-lang.github.io/rust-clippy/master/)
