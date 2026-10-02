# M0 — Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Install the Rust toolchain and give the repository its final shape (a three-crate Cargo
workspace with shared lints, tests, a license, a public GitHub repository and CI), so that
`cargo run -p platformer` prints a greeting that comes from the engine.

**Architecture:** A virtual-manifest Cargo workspace with `crates/ecs` (lib, no dependencies),
`crates/engine` (lib, depends on `ecs`) and `games/platformer` (bin, depends on `engine`). Shared
metadata, dependency declarations and lints live in the root `Cargo.toml`. The only code is
`engine::greeting()`, which crosses both crate boundaries and is covered by a unit test, a doc test
and an integration test that runs the binary.

**Tech Stack:** Rust stable (rustup), Cargo workspaces, rustfmt, clippy, GitHub Actions, GitHub CLI
(`gh`).

## Global Constraints

- Run every command from the repository root, `/Users/alekseinovikov/rs-engine`.
- Toolchain: Rust `stable` channel with `rustfmt` and `clippy`, pinned by `rust-toolchain.toml`;
  installed with the official rustup script.
- Workspace: edition `2024`, resolver `"3"`, version `0.1.0`, `publish = false`.
- Crates: `ecs` at `crates/ecs` (lib), `engine` at `crates/engine` (lib), `platformer` at
  `games/platformer` (bin). Dependencies point downward only: `platformer` → `engine` → `ecs`.
- No external crates in M0.
- Workspace lints: `unsafe_code = "deny"`, `missing_docs = "warn"`.
  `cargo clippy --workspace --all-targets -- -D warnings` must pass, and code is formatted with
  `cargo fmt --all`.
- Every module starts with `//!` docs; comments explain *why*.
- Everything in the repository is in English. Every commit message ends with the trailer
  `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- License: `MIT OR Apache-2.0`; MIT copyright line `Copyright (c) 2026 Aleksei Novikov`.
- GitHub: public repository `alekseinovikov/rs-engine`; CI on `ubuntu-latest` and `macos-latest`
  with `actions/checkout@v7` and `Swatinem/rust-cache@v2`.
- Git: all M0 work happens on the branch `milestone/m0-foundation` and reaches `main` through a
  pull request, merged with a merge commit after the user's review; the tag `m0-foundation` goes on
  the merge commit.
- Progress tracking: tick each step's checkbox in this file as it is completed and include this
  file in the task's commit.

## File Structure

| Path | Responsibility |
|---|---|
| `rust-toolchain.toml` | Pins the toolchain channel and components for rustup |
| `Cargo.toml` | Workspace: members, shared package metadata, shared dependencies, shared lints |
| `Cargo.lock` | Exact dependency versions (generated, committed) |
| `crates/ecs/Cargo.toml`, `crates/ecs/src/lib.rs` | The future ECS; for now crate docs and `VERSION` |
| `crates/engine/Cargo.toml`, `crates/engine/src/lib.rs` | The engine; for now crate docs, `VERSION`, `greeting()` and its tests |
| `games/platformer/Cargo.toml`, `games/platformer/src/main.rs` | The game binary; prints the greeting |
| `games/platformer/tests/startup.rs` | Integration test that runs the binary |
| `LICENSE-MIT`, `LICENSE-APACHE`, `README.md` | License texts and the project overview |
| `.github/workflows/ci.yml` | CI: fmt, clippy and tests on Linux and macOS |
| `docs/learning/00-foundation.md` | The learning note for M0 |
| `CLAUDE.md`, `ROADMAP.md`, `docs/superpowers/specs/2026-10-02-engine-mvp-design.md` | Status and decision updates when the milestone closes |

---

### Task 1: Install the Rust toolchain

**Files:**
- Modify: `ROADMAP.md` (M0 status, milestone process)
- Modify: `CLAUDE.md` (session start, milestone workflow)

**Interfaces:**
- Consumes: nothing.
- Produces: the branch `milestone/m0-foundation`; `rustup`, `rustc`, `cargo`, `cargo fmt` and
  `cargo clippy` on `PATH` (through `~/.cargo/bin`).

- [x] **Step 1: Create the milestone branch**

Run: `git switch -c milestone/m0-foundation`

Expected: `Switched to a new branch 'milestone/m0-foundation'`. The branch name differs from the
future tag `m0-foundation` on purpose: a branch and a tag with the same name make
`git checkout m0-foundation` ambiguous.

- [x] **Step 2: Mark M0 as in progress**

In `ROADMAP.md`, replace

```markdown
**Current milestone:** M0 — Foundation (not started)
```

with

```markdown
**Current milestone:** M0 — Foundation (in progress)
```

and in the table row that starts with `| M0 | Foundation |`, replace the final `⬜` with `🟨`.

- [x] **Step 3: Record the branch-per-milestone workflow**

In `ROADMAP.md`, replace the whole `## How we work through a milestone` section (from the heading
down to, not including, the `---` line above `## M0 — Foundation`) with:

```markdown
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

```

In `CLAUDE.md`, replace the `## Session start` and `## Milestone workflow` sections (from
`## Session start` down to, not including, `## Architecture`) with:

```markdown
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

```

- [x] **Step 4: Install rustup and the stable toolchain**

Run:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
```

`--proto '=https' --tlsv1.2` refuses anything but HTTPS with TLS 1.2 or newer. `-sSf` hides the
progress bar but still shows errors and fails on HTTP errors. `-y` accepts the defaults: the stable
toolchain with the default profile (rustc, cargo, rust-std, rust-docs, rustfmt, clippy), installed
into `~/.rustup` and `~/.cargo`, plus a line in the shell profile files (for zsh, `~/.zshenv`) that
adds `~/.cargo/bin` to `PATH`.

Expected: the output ends with `Rust is installed now. Great!`

- [x] **Step 5: Verify the toolchain**

Run:

```bash
source "$HOME/.cargo/env"
rustup --version && rustc --version && cargo --version && cargo fmt --version && cargo clippy --version
```

Expected: five version lines, with `rustc` 1.85 or newer (edition 2024 requires 1.85). New shells
pick up `PATH` automatically; if `cargo` is not found in one, run `source "$HOME/.cargo/env"`.

- [x] **Step 6: Commit**

```bash
git add ROADMAP.md CLAUDE.md docs/superpowers/plans/2026-10-02-m0-foundation.md
git commit -m "Start milestone M0" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: Cargo workspace and the `ecs` crate

**Files:**
- Create: `rust-toolchain.toml`
- Create: `Cargo.toml`
- Create: `crates/ecs/Cargo.toml`
- Create: `crates/ecs/src/lib.rs`
- Generated: `Cargo.lock`

**Interfaces:**
- Consumes: the toolchain from Task 1.
- Produces: `pub const ecs::VERSION: &str` (the `ecs` package version, today `"0.1.0"`), and a root
  `Cargo.toml` with `[workspace.package]` and `[workspace.lints.rust]` that Tasks 3 and 4 extend.

- [x] **Step 1: Pin the toolchain**

Create `rust-toolchain.toml`:

```toml
# Tells rustup which toolchain to use inside this repository. rustup reads this file
# automatically and installs whatever is missing the first time `cargo` runs here.
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
```

- [x] **Step 2: Create the workspace manifest**

Create `Cargo.toml`:

```toml
# The workspace manifest. It has no [package] section of its own (a "virtual manifest"):
# it lists the member crates and the settings they share.
[workspace]
resolver = "3"
members = [
    "crates/ecs",
]

# Package metadata that members inherit with `key.workspace = true`.
[workspace.package]
version = "0.1.0"
edition = "2024"
license = "MIT OR Apache-2.0"
repository = "https://github.com/alekseinovikov/rs-engine"
publish = false

# Lints shared by all members; each member opts in with `[lints] workspace = true`.
[workspace.lints.rust]
# Our own code never uses `unsafe`. The level is `deny` rather than `forbid` so that a
# narrowly scoped, documented `#[allow(unsafe_code)]` stays possible if a dependency's
# derive macro ever generates an `unsafe impl`.
unsafe_code = "deny"
# Every public item gets a doc comment: in a learning codebase, docs are half the point.
missing_docs = "warn"
```

- [x] **Step 3: Create the `ecs` crate**

Create `crates/ecs/Cargo.toml`:

```toml
[package]
name = "ecs"
description = "A minimal entity-component system, written from scratch for learning."
version.workspace = true
edition.workspace = true
license.workspace = true
repository.workspace = true
publish.workspace = true

[lints]
workspace = true
```

Create `crates/ecs/src/lib.rs`:

```rust
//! A minimal entity-component system (ECS), written from scratch for learning.
//!
//! An ECS splits a game world into three kinds of things:
//!
//! - **Entities** are plain identifiers ("thing number 42"). They own no data and no behavior.
//! - **Components** are plain data attached to entities: a position, a sprite, a velocity.
//! - **Systems** are functions that run over every entity that has a particular set of
//!   components, for example "move everything that has both a position and a velocity".
//!
//! The crate is intentionally empty for now: it is built in milestone M5 (see `ROADMAP.md`).
//! It exists from day one so that the workspace has its final shape and the dependency
//! `engine` → `ecs` is already in place.

/// The version of this crate, read from its `Cargo.toml` at compile time.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
```

- [x] **Step 4: Build and check**

Run:

```bash
cargo build --workspace
cargo fmt --all
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
```

Expected: `Compiling ecs v0.1.0 (...)` followed by `Finished`; `cargo fmt --all --check` prints
nothing; clippy finishes without warnings. A `Cargo.lock` file appears in the repository root.

- [x] **Step 5: Commit**

```bash
git add rust-toolchain.toml Cargo.toml Cargo.lock crates/ecs docs/superpowers/plans/2026-10-02-m0-foundation.md
git commit -m "Add the Cargo workspace and the ecs crate" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: The `engine` crate and `greeting()`

**Files:**
- Modify: `Cargo.toml` (members, new `[workspace.dependencies]` section)
- Create: `crates/engine/Cargo.toml`
- Create: `crates/engine/src/lib.rs` (implementation and unit test)

**Interfaces:**
- Consumes: `ecs::VERSION: &str` (Task 2).
- Produces: `pub const engine::VERSION: &str` and `pub fn engine::greeting() -> String`, which
  returns `"rs-engine <engine version> (ecs <ecs version>)"`, today
  `"rs-engine 0.1.0 (ecs 0.1.0)"`.

- [x] **Step 1: Register the crate in the workspace**

In `Cargo.toml`, replace

```toml
members = [
    "crates/ecs",
]
```

with

```toml
members = [
    "crates/ecs",
    "crates/engine",
]
```

and insert this section between the `[workspace.package]` section and the
`# Lints shared by all members` comment:

```toml
# Dependencies shared by all members. A member opts in with `name.workspace = true`, so
# every version (or path) is written in exactly one place.
[workspace.dependencies]
ecs = { path = "crates/ecs" }
```

- [x] **Step 2: Create the crate manifest**

Create `crates/engine/Cargo.toml`:

```toml
[package]
name = "engine"
description = "A small 2D game engine framework, written from scratch for learning."
version.workspace = true
edition.workspace = true
license.workspace = true
repository.workspace = true
publish.workspace = true

[dependencies]
ecs.workspace = true

[lints]
workspace = true
```

- [x] **Step 3: Write the failing test**

Create `crates/engine/src/lib.rs`:

```rust
//! The rs-engine framework, written from scratch for learning.
//!
//! The engine is a *framework*: it owns the window and the game loop and calls back into the
//! game. Milestone by milestone (see `ROADMAP.md`) it grows a game loop, input handling, a GPU
//! sprite renderer, assets, tilemaps, physics, animation, text, audio and scenes.
//!
//! For now it only provides [`greeting`], which proves that the workspace crates are wired
//! together: `platformer` → `engine` → `ecs`.

/// The version of this crate, read from its `Cargo.toml` at compile time.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greeting_names_the_engine_and_its_ecs() {
        let text = greeting();
        let engine_part = format!("rs-engine {VERSION} ");
        let ecs_part = format!("(ecs {})", ecs::VERSION);

        assert!(text.starts_with(&engine_part), "got: {text}");
        assert!(text.ends_with(&ecs_part), "got: {text}");
    }
}
```

- [x] **Step 4: Run the test to verify it fails**

Run: `cargo test -p engine`

Expected: compilation fails with ``error[E0425]: cannot find function `greeting` in this scope``.

- [x] **Step 5: Implement `greeting()`**

In `crates/engine/src/lib.rs`, insert between the `VERSION` constant and the `#[cfg(test)]` line:

```rust
/// Returns a one-line greeting that names the engine and the ECS it is built on.
///
/// The versions come from `env!("CARGO_PKG_VERSION")`, which Cargo fills in at compile time, so
/// they always match the crates that were actually compiled.
///
/// # Example
///
/// ```
/// let text = engine::greeting();
/// assert!(text.starts_with("rs-engine "));
/// ```
pub fn greeting() -> String {
    format!("rs-engine {VERSION} (ecs {})", ecs::VERSION)
}
```

- [x] **Step 6: Run the tests to verify they pass**

Run: `cargo test -p engine`

Expected: `test tests::greeting_names_the_engine_and_its_ecs ... ok` among the unit tests, and a
line like `test crates/engine/src/lib.rs - greeting (line 20) ... ok` under `Doc-tests engine`.

- [x] **Step 7: Format, lint and commit**

```bash
cargo fmt --all && cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
git add Cargo.toml Cargo.lock crates/engine docs/superpowers/plans/2026-10-02-m0-foundation.md
git commit -m "Add the engine crate with a greeting" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

Expected: no formatting diff and no clippy warnings before the commit.

---

### Task 4: The `platformer` binary

**Files:**
- Modify: `Cargo.toml` (members, `[workspace.dependencies]`)
- Create: `games/platformer/Cargo.toml`
- Create: `games/platformer/src/main.rs`
- Test: `games/platformer/tests/startup.rs`

**Interfaces:**
- Consumes: `engine::greeting() -> String` (Task 3).
- Produces: the binary `platformer`, which prints `engine::greeting()` and a newline to stdout and
  exits with status 0.

- [ ] **Step 1: Register the package in the workspace**

In `Cargo.toml`, replace

```toml
members = [
    "crates/ecs",
    "crates/engine",
]
```

with

```toml
members = [
    "crates/ecs",
    "crates/engine",
    "games/platformer",
]
```

and in `[workspace.dependencies]`, add below the `ecs = ...` line:

```toml
engine = { path = "crates/engine" }
```

- [ ] **Step 2: Create the package with an empty `main`**

Create `games/platformer/Cargo.toml`:

```toml
[package]
name = "platformer"
description = "The rs-engine MVP demo: a small platformer."
version.workspace = true
edition.workspace = true
license.workspace = true
repository.workspace = true
publish.workspace = true

[dependencies]
engine.workspace = true

[lints]
workspace = true
```

Create `games/platformer/src/main.rs`:

```rust
//! The MVP demo game: a small platformer built on rs-engine.
//!
//! For now it only prints the engine's greeting, which proves that the workspace is wired
//! together. It becomes a real game from milestone M6 on (see `ROADMAP.md`).

fn main() {}
```

- [ ] **Step 3: Write the failing integration test**

Create `games/platformer/tests/startup.rs`:

```rust
//! Integration test: runs the compiled game binary and checks what it prints.
//!
//! Integration tests live in `tests/` and see the package only from the outside, the way a
//! user does. This one becomes obsolete once the game opens a window (M6), because running
//! the binary would then block until the window is closed.

use std::process::Command;

#[test]
fn prints_the_engine_greeting() {
    // Cargo builds the binary before running integration tests and passes its path here.
    let output = Command::new(env!("CARGO_BIN_EXE_platformer"))
        .output()
        .expect("failed to run the platformer binary");

    assert!(output.status.success(), "exit status: {}", output.status);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), engine::greeting());
}
```

- [ ] **Step 4: Run the test to verify it fails**

Run: `cargo test -p platformer`

Expected: `prints_the_engine_greeting` FAILS with ``assertion `left == right` failed``,
`left: ""` and `right: "rs-engine 0.1.0 (ecs 0.1.0)"`.

- [ ] **Step 5: Print the greeting**

In `games/platformer/src/main.rs`, replace `fn main() {}` with:

```rust
fn main() {
    println!("{}", engine::greeting());
}
```

- [ ] **Step 6: Run the tests to verify they pass**

Run: `cargo test -p platformer`
Expected: `test prints_the_engine_greeting ... ok`.

Run: `cargo run -p platformer`
Expected output: `rs-engine 0.1.0 (ecs 0.1.0)`.

- [ ] **Step 7: Full check and commit**

```bash
cargo fmt --all && cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
git add Cargo.toml Cargo.lock games/platformer docs/superpowers/plans/2026-10-02-m0-foundation.md
git commit -m "Add the platformer binary that prints the engine greeting" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

Expected: no formatting diff, no clippy warnings, every test passes.

---

### Task 5: License and README

**Files:**
- Create: `LICENSE-MIT`
- Create: `LICENSE-APACHE`
- Create: `README.md`

**Interfaces:**
- Consumes: `license = "MIT OR Apache-2.0"` in `[workspace.package]` (Task 2); the crate names and
  paths from Tasks 2–4.
- Produces: license files that match the Cargo metadata; a README that later milestones extend.

- [ ] **Step 1: Write the license files**

The license texts come from GitHub's licenses API (the same texts GitHub's license picker uses). The
MIT template has `[year]` and `[fullname]` placeholders; the Apache-2.0 text is used verbatim
(its appendix placeholders are part of the license text).

```bash
gh api licenses/mit --jq .body | sed -e 's/\[year\]/2026/' -e 's/\[fullname\]/Aleksei Novikov/' > LICENSE-MIT
gh api licenses/apache-2.0 --jq .body > LICENSE-APACHE
```

- [ ] **Step 2: Verify the license files**

Run:

```bash
sed -n '3p' LICENSE-MIT
grep -c '\[year\]\|\[fullname\]' LICENSE-MIT
head -3 LICENSE-APACHE
```

Expected: `Copyright (c) 2026 Aleksei Novikov`, then `0`, then the Apache header lines containing
`Apache License` and `Version 2.0, January 2004`.

- [ ] **Step 3: Write the README**

Create `README.md`:

````markdown
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
````

- [ ] **Step 4: Commit**

```bash
git add LICENSE-MIT LICENSE-APACHE README.md docs/superpowers/plans/2026-10-02-m0-foundation.md
git commit -m "Add the README and the MIT OR Apache-2.0 license" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 6: CI and the GitHub repository

**Files:**
- Create: `.github/workflows/ci.yml`

**Interfaces:**
- Consumes: `rust-toolchain.toml` (Task 2); the check commands from Tasks 2–4.
- Produces: the public repository `https://github.com/alekseinovikov/rs-engine` with the remote
  `origin`, a `CI` workflow that runs on every push to `main` and on pull requests, and the open
  pull request `milestone/m0-foundation` → `main`.

- [ ] **Step 1: Write the workflow**

Create `.github/workflows/ci.yml`:

```yaml
# Continuous integration: checks every push to main and every pull request on Linux and macOS.
name: CI

on:
  push:
    branches: [main]
  pull_request:

env:
  CARGO_TERM_COLOR: always

jobs:
  check:
    name: Check (${{ matrix.os }})
    runs-on: ${{ matrix.os }}
    strategy:
      # Let both operating systems finish even if one fails, to see every problem at once.
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest]
    steps:
      - name: Check out the repository
        uses: actions/checkout@v7

      # rustup is preinstalled on GitHub runners. Without arguments, this installs the
      # toolchain and the components listed in rust-toolchain.toml.
      - name: Install the Rust toolchain
        run: rustup toolchain install

      # Caches ~/.cargo and target/ between runs, keyed by Cargo.lock and the toolchain.
      - name: Cache build artifacts
        uses: Swatinem/rust-cache@v2

      - name: Check formatting
        run: cargo fmt --all --check

      - name: Clippy
        run: cargo clippy --workspace --all-targets -- -D warnings

      - name: Tests
        run: cargo test --workspace
```

- [ ] **Step 2: Commit the workflow**

```bash
git add .github/workflows/ci.yml docs/superpowers/plans/2026-10-02-m0-foundation.md
git commit -m "Add CI for Linux and macOS" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

- [ ] **Step 3: Create the GitHub repository and push both branches**

```bash
gh repo create alekseinovikov/rs-engine --public --description "A small 2D game engine in Rust, written from scratch for learning" --source . --remote origin
git push -u origin main
git push -u origin milestone/m0-foundation
```

Expected: `✓ Created repository alekseinovikov/rs-engine on GitHub` and `✓ Added remote ...`; both
pushes succeed. Pushing `main` starts no CI run, because so far the workflow file exists only on
the milestone branch.

- [ ] **Step 4: Open the pull request**

````bash
gh pr create --base main --head milestone/m0-foundation --title "M0: Foundation" --body-file - <<'EOF'
Milestone M0 from ROADMAP.md: the Rust toolchain and the final shape of the repository.

## What's inside

- A Cargo workspace (edition 2024, resolver 3) with three crates: `ecs` (lib), `engine` (lib) and
  `platformer` (bin), plus shared metadata, dependencies and lints.
- `engine::greeting()` crosses both crate boundaries and is covered by a unit test, a doc test and
  an integration test that runs the binary.
- `rust-toolchain.toml`, README, the MIT OR Apache-2.0 license, CI on Linux and macOS.
- The learning note `docs/learning/00-foundation.md`.

## How to check

```bash
cargo run -p platformer   # prints: rs-engine 0.1.0 (ecs 0.1.0)
cargo test --workspace
```

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
````

Expected: the command prints the pull request URL. Opening the pull request starts the `CI`
workflow (`pull_request` trigger).

- [ ] **Step 5: Confirm that CI started**

Bind the pull request to the session and read its checks once: in the Claude desktop app with the
PR tools (`get_status`, then `bind_pr` with the PR URL if the PR is not reported yet); elsewhere
with `gh pr checks milestone/m0-foundation`. Expected: the checks `Check (ubuntu-latest)` and
`Check (macos-latest)` are queued or running. Do not wait for them here: Task 8 reads the result.

---

### Task 7: Learning note

**Files:**
- Create: `docs/learning/00-foundation.md`

**Interfaces:**
- Consumes: everything built in Tasks 1–6 (the note links to those files).
- Produces: the M0 learning note; later notes follow the same structure (What we built, Key
  concepts, Where to look in the code, Experiments to try, Further reading).

- [ ] **Step 1: Write the note**

Create `docs/learning/00-foundation.md`:

````markdown
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
````

- [ ] **Step 2: Commit and push**

```bash
git add docs/learning/00-foundation.md docs/superpowers/plans/2026-10-02-m0-foundation.md
git commit -m "Add the M0 learning note" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git push
```

---

### Task 8: Close the milestone

**Files:**
- Modify: `CLAUDE.md`
- Modify: `ROADMAP.md`
- Modify: `docs/superpowers/specs/2026-10-02-engine-mvp-design.md`

**Interfaces:**
- Consumes: everything above, including the open pull request.
- Produces: M0 marked done and M1 as the current milestone; the pull request merged into `main`;
  the tag `m0-foundation` on the merge commit, pushed to GitHub.

- [ ] **Step 1: Run the full local verification**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo run -p platformer
```

Expected: no formatting diff, no clippy warnings, all tests pass (unit, integration and doc tests),
and the last line printed is `rs-engine 0.1.0 (ecs 0.1.0)`.

- [ ] **Step 2: Update CLAUDE.md**

1. Replace `## Commands (available after M0)` with `## Commands`.
2. Replace

   ```markdown
   - Public items are documented (`missing_docs` warns in `engine` and `ecs`).
   ```

   with

   ```markdown
   - Public items are documented. Workspace lints in the root `Cargo.toml`: `missing_docs` warns
     and `unsafe_code` is denied in every crate.
   ```

3. In `## Project`, add after the `- Design and decision log: ...` line:

   ```markdown
   - Repository: https://github.com/alekseinovikov/rs-engine (public). CI runs on every push to
     `main` and on pull requests.
   ```

- [ ] **Step 3: Update ROADMAP.md**

1. Replace `**Current milestone:** M0 — Foundation (in progress)` with
   `**Current milestone:** M1 — Window, game loop, input (not started)`.
2. In the table row that starts with `| M0 | Foundation |`, replace `🟨` with `✅`.
3. In the M0 section, replace

   ```markdown
   - `README.md` and a license (decide with the user).
   ```

   with

   ```markdown
   - `README.md` and the MIT OR Apache-2.0 dual license.
   ```

4. In the M0 section, replace

   ```markdown
   - GitHub Actions CI: fmt, clippy, tests, build (Ubuntu and macOS). Create a GitHub remote only if
     the user wants one.
   ```

   with

   ```markdown
   - GitHub Actions CI: fmt, clippy, tests, build (Ubuntu and macOS) in the public repository
     `alekseinovikov/rs-engine`.
   ```

5. In the M4 section, replace `nearest-neighbour sampler` with `nearest-neighbor sampler`.

- [ ] **Step 4: Update the design spec**

In `docs/superpowers/specs/2026-10-02-engine-mvp-design.md`:

1. §5: replace `Nearest-neighbour sampling everywhere.` with `Nearest-neighbor sampling everywhere.`
2. §8: replace

   ```markdown
   - CI on GitHub Actions, active once a remote exists: `cargo fmt --check`,
     `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, a build of
     all targets. Ubuntu and macOS runners.
   ```

   with

   ```markdown
   - CI on GitHub Actions in the public repository `alekseinovikov/rs-engine`: `cargo fmt --check`,
     `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` (which also
     builds every example). Ubuntu and macOS runners.
   ```

3. §9: replace `autotiled from their neighbours.` with `autotiled from their neighbors.`
4. §11: insert below the `├── ROADMAP.md             milestones and status` line:

   ```
   ├── README.md, LICENSE-*   overview and the dual license
   ```

5. §12: replace

   ```markdown
   - `missing_docs` is a warning in `engine` and `ecs`; CI treats warnings as errors.
   ```

   with

   ```markdown
   - Workspace lints in the root `Cargo.toml`: `missing_docs` warns and `unsafe_code` is denied in
     every crate; CI treats warnings as errors.
   ```

6. §14: delete the line
   `- License (suggested: MIT OR Apache-2.0) and whether to create a GitHub remote: M0.`
7. §15: append these rows to the decision log table:

   ```markdown
   | 2026-10-02 | Rust installed with the official rustup script; `rust-toolchain.toml` follows the stable channel. |
   | 2026-10-02 | License: MIT OR Apache-2.0. Public GitHub repository `alekseinovikov/rs-engine`; CI on Ubuntu and macOS. |
   | 2026-10-02 | Workspace lints: `unsafe_code = "deny"` and `missing_docs = "warn"` in every crate. |
   | 2026-10-02 | Git workflow: one branch per milestone (`milestone/mN-<name>`), merged into `main` through a pull request with a merge commit; milestone tags go on the merge commits. |
   ```

- [ ] **Step 5: Commit and push**

```bash
git add CLAUDE.md ROADMAP.md docs/superpowers/specs/2026-10-02-engine-mvp-design.md docs/superpowers/plans/2026-10-02-m0-foundation.md
git commit -m "Complete milestone M0" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git push
```

- [ ] **Step 6: Check CI on the pull request**

Read the pull request's checks once (desktop app: PR tool `get_status`; elsewhere:
`gh pr checks milestone/m0-foundation`). Expected: `Check (ubuntu-latest)` and
`Check (macos-latest)` pass. If they are still running, continue with Step 7 and read them again
when the user answers. If a check failed, read its log with `gh run view <run-id> --log-failed`,
fix the cause, commit and push.

- [ ] **Step 7: User review**

Ask the user to run `cargo run -p platformer` and `cargo test --workspace`, read the code and
`docs/learning/00-foundation.md`, try a few experiments, and review the pull request on GitHub.
Apply requested changes on the branch (commit and push) before continuing.

- [ ] **Step 8: Merge and tag**

Only after the user's OK and with green CI:

```bash
gh pr merge milestone/m0-foundation --merge --delete-branch
git switch main
git pull --ff-only
git tag -a m0-foundation -m "M0: Foundation"
git push origin m0-foundation
```

Expected: the pull request is merged with a merge commit, the branch is deleted locally and on
GitHub, local `main` matches `origin/main`, and the tag `m0-foundation` points at the merge commit.
The merge into `main` starts a CI run there.

Steps 6–8 happen after the last commit on the branch, so their checkboxes stay unticked in the
merged plan; ROADMAP.md and the tag are the record that M0 is done.
