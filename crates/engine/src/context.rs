//! What the game receives in every `update` and `draw` call.
//!
//! The subsystems are *public fields*, not getter methods. With fields, the borrow checker sees
//! that `ctx.input` and `ctx.time` are separate places, so a game can read one while mutating
//! another. Methods like `ctx.input()` would borrow the whole `Context` at once. The context
//! grows milestone by milestone: `gfx` arrives in M2, `assets` in M6, `audio` in M10.

use crate::config::Config;
use crate::input::Input;
use crate::pacing;
use crate::time::Time;

/// The engine's subsystems, lent to the game for the duration of a callback.
#[derive(Debug)]
pub struct Context {
    /// Keyboard and mouse state for the current tick.
    pub input: Input,
    /// The fixed step, frame time and FPS.
    pub time: Time,
    max_fps: Option<u32>,
    quit_requested: bool,
}

// Used by app.rs from Task 7 on; until then only tests call these.
#[allow(dead_code)]
impl Context {
    /// Creates the context for a fresh run.
    pub(crate) fn new(config: &Config) -> Self {
        Self {
            input: Input::new(),
            time: Time::default(),
            max_fps: config.max_fps,
            quit_requested: false,
        }
    }

    /// Asks the engine to close the window and leave `engine::run` after the current frame.
    pub fn quit(&mut self) {
        self.quit_requested = true;
    }

    /// Whether the game called [`Context::quit`].
    pub(crate) fn quit_requested(&self) -> bool {
        self.quit_requested
    }

    /// The current frame limit; `None` means no limit.
    pub fn max_fps(&self) -> Option<u32> {
        self.max_fps
    }

    /// Changes the frame limit from the next frame on. `None` removes the limit.
    ///
    /// # Panics
    ///
    /// Panics on `Some(0)`.
    pub fn set_max_fps(&mut self, max_fps: Option<u32>) {
        // Validate now, so the panic points at the caller rather than at the engine's loop.
        pacing::frame_period(max_fps);
        self.max_fps = max_fps;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quit_sets_the_flag() {
        let mut ctx = Context::new(&Config::default());
        assert!(!ctx.quit_requested());
        ctx.quit();
        assert!(ctx.quit_requested());
    }

    #[test]
    fn the_frame_limit_starts_from_the_config_and_can_change() {
        let config = Config {
            max_fps: Some(60),
            ..Default::default()
        };
        let mut ctx = Context::new(&config);
        assert_eq!(ctx.max_fps(), Some(60));
        ctx.set_max_fps(None);
        assert_eq!(ctx.max_fps(), None);
    }

    #[test]
    #[should_panic(expected = "max_fps must be at least 1")]
    fn a_zero_frame_limit_is_rejected() {
        Context::new(&Config::default()).set_max_fps(Some(0));
    }
}
