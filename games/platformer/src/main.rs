//! The MVP demo game: a small platformer built on rs-engine.
//!
//! For now it only prints the engine's greeting, which proves that the workspace is wired
//! together. It becomes a real game from milestone M6 on (see `ROADMAP.md`).

fn main() {
    println!("{}", engine::greeting());
}
