//! Start-up settings for the engine.

/// How the engine sets up the window and the loop. Start from `Config::default()` and override
/// what you need with struct update syntax:
///
/// ```
/// let config = engine::Config {
///     title: "My game".to_string(),
///     max_fps: Some(60),
///     ..Default::default()
/// };
/// assert_eq!(config.width, 960);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    /// The window title. The engine appends the current FPS to it.
    pub title: String,
    /// The window's inner width in logical pixels (physical pixels divided by the display
    /// scale factor, so the window looks the same size on Retina and regular displays).
    pub width: u32,
    /// The window's inner height in logical pixels.
    pub height: u32,
    /// The frame limit, or `None` to render frames back to back. Can be changed at runtime
    /// with `Context::set_max_fps`. Must not be `Some(0)`.
    pub max_fps: Option<u32>,
    /// Whether Esc closes the window. Games that use Esc themselves (for a pause menu) turn
    /// this off.
    pub quit_on_escape: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            title: "rs-engine".to_string(),
            width: 960,
            height: 540,
            max_fps: None,
            quit_on_escape: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_is_a_960_by_540_window_without_a_frame_limit() {
        let config = Config::default();
        assert_eq!((config.width, config.height), (960, 540));
        assert_eq!(config.max_fps, None);
        assert!(config.quit_on_escape);
    }
}
