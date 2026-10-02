//! What the game receives in every `update` and `draw` call.
//!
//! The subsystems are *public fields*, not getter methods. With fields, the borrow checker sees
//! that `ctx.input` and `ctx.gfx` are separate places, so a game can read one while mutating
//! another. Methods like `ctx.input()` would borrow the whole `Context` at once. The context
//! grows milestone by milestone: `assets` arrives in M6, `audio` in M10.
//!
//! A `Context` can only be built once a window and a GPU exist (because of `gfx`), so the
//! engine creates it in `resumed`, and unit tests cannot build one. The loop settings the game
//! changes through the context therefore live in a small struct of their own, [`LoopControl`],
//! which the tests check directly.

use crate::config::Config;
use crate::gfx::Gfx;
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
    /// The renderer: draw calls, the clear color, vsync.
    pub gfx: Gfx,
    control: LoopControl,
}

impl Context {
    /// Creates the context for a fresh run, around a renderer that is already set up.
    pub(crate) fn new(config: &Config, gfx: Gfx) -> Self {
        Self {
            input: Input::new(),
            time: Time::default(),
            gfx,
            control: LoopControl::new(config),
        }
    }

    /// Asks the engine to close the window and leave `engine::run` after the current frame.
    pub fn quit(&mut self) {
        self.control.quit_requested = true;
    }

    /// Whether the game called [`Context::quit`].
    pub(crate) fn quit_requested(&self) -> bool {
        self.control.quit_requested
    }

    /// The current frame limit; `None` means no limit.
    pub fn max_fps(&self) -> Option<u32> {
        self.control.max_fps
    }

    /// Changes the frame limit from the next frame on. `None` removes the limit.
    ///
    /// # Panics
    ///
    /// Panics on `Some(0)`.
    pub fn set_max_fps(&mut self, max_fps: Option<u32>) {
        self.control.set_max_fps(max_fps);
    }
}

/// The requests the game makes to the loop: quitting and the frame limit.
#[derive(Debug)]
struct LoopControl {
    max_fps: Option<u32>,
    quit_requested: bool,
}

impl LoopControl {
    fn new(config: &Config) -> Self {
        Self {
            max_fps: config.max_fps,
            quit_requested: false,
        }
    }

    fn set_max_fps(&mut self, max_fps: Option<u32>) {
        // Validate now, so the panic points at the caller rather than at the engine's loop.
        pacing::frame_period(max_fps);
        self.max_fps = max_fps;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_is_requested_at_the_start() {
        let control = LoopControl::new(&Config::default());
        assert!(!control.quit_requested);
    }

    #[test]
    fn the_frame_limit_starts_from_the_config_and_can_change() {
        let config = Config {
            max_fps: Some(60),
            ..Default::default()
        };
        let mut control = LoopControl::new(&config);
        assert_eq!(control.max_fps, Some(60));
        control.set_max_fps(None);
        assert_eq!(control.max_fps, None);
    }

    #[test]
    #[should_panic(expected = "max_fps must be at least 1")]
    fn a_zero_frame_limit_is_rejected() {
        LoopControl::new(&Config::default()).set_max_fps(Some(0));
    }
}
