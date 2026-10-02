//! The rs-engine framework, written from scratch for learning.
//!
//! The engine is a *framework*: it owns the window and the game loop and calls back into the
//! game. Milestone by milestone (see `ROADMAP.md`) it grows a game loop, input handling, a GPU
//! sprite renderer, assets, tilemaps, physics, animation, text, audio and scenes.
//!
//! For now it only provides [`greeting`], which proves that the workspace crates are wired
//! together: `platformer` → `engine` → `ecs`.

mod fps;
mod pacing;
mod time;

pub use time::Time;

/// The version of this crate, read from its `Cargo.toml` at compile time.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

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
