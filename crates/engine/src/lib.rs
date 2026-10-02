//! The rs-engine framework, written from scratch for learning.
//!
//! The engine is a *framework*: it owns the window and the game loop and calls back into the
//! game, which implements [`Game`]. Start it with [`run`]:
//!
//! ```no_run
//! use engine::{Config, Context, Game};
//!
//! struct Hello;
//!
//! impl Game for Hello {
//!     fn update(&mut self, ctx: &mut Context) {}
//!     fn draw(&mut self, ctx: &mut Context) {}
//! }
//!
//! fn main() -> Result<(), engine::Error> {
//!     engine::run(Config::default(), |_ctx| Hello)
//! }
//! ```
//!
//! Milestone by milestone (see `ROADMAP.md`) the engine grows a GPU sprite renderer, assets,
//! tilemaps, physics, animation, text, audio and scenes.

mod app;
mod config;
mod context;
mod error;
mod fps;
mod gfx;
mod input;
mod pacing;
mod platform;
mod time;

pub use config::Config;
pub use context::Context;
pub use error::Error;
pub use gfx::{Color, Gfx, Vertex};
pub use input::{Input, Key, MouseButton};
pub use time::Time;

use winit::event_loop::EventLoop;

/// A game driven by the engine.
///
/// The engine calls [`Game::update`] zero or more times per frame, always with the same fixed
/// step, and then [`Game::draw`] exactly once. Keep all game logic in `update`: then it runs at
/// the same speed on every computer, whatever the frame rate.
pub trait Game {
    /// Advances the simulation by exactly one fixed step (`ctx.time.fixed_dt()` seconds).
    fn update(&mut self, ctx: &mut Context);
    /// Draws the current state. Called once per frame, after the frame's updates.
    fn draw(&mut self, ctx: &mut Context);
}

/// Opens a window and runs the game loop until the window closes.
///
/// `make_game` runs once, after the window exists, and builds the game. It receives the
/// [`Context`], so the game can set up its renderer state (and, from M6 on, load its assets)
/// there.
///
/// # Errors
///
/// Returns [`Error::EventLoop`] if the OS event loop cannot be created, [`Error::Window`] if
/// the window cannot be opened, and one of the GPU variants ([`Error::CreateSurface`],
/// [`Error::RequestAdapter`], [`Error::RequestDevice`], [`Error::UnsupportedSurface`]) if the
/// renderer cannot start.
pub fn run<G, F>(config: Config, make_game: F) -> Result<(), Error>
where
    G: Game,
    F: FnOnce(&mut Context) -> G,
{
    let event_loop = EventLoop::new()?;
    let mut app = app::App::new(config, make_game);
    event_loop.run_app(&mut app)?;
    match app.take_error() {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

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
