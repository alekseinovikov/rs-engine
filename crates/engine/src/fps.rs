//! Measuring frames per second.
//!
//! A per-frame value (`1 / frame_dt`) jumps around too much to read, so the counter averages:
//! it counts frames over a window of time (half a second by default) and reports
//! `frames / elapsed` once per window. Averaging over *time* rather than over a fixed number of
//! frames gives a steady update rate whether the game runs at 30 or 3000 FPS.

use std::time::Duration;

/// Counts frames and reports their average rate once per window of time.
// Used by app.rs from Task 7 on; until then only tests call these.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct FpsCounter {
    window: Duration,
    frames: u32,
    elapsed: Duration,
}

// Used by app.rs from Task 7 on; until then only tests call these.
#[allow(dead_code)]
impl FpsCounter {
    /// How often a new FPS value is reported.
    pub(crate) const DEFAULT_WINDOW: Duration = Duration::from_millis(500);

    /// Creates a counter that reports once per `window`.
    pub(crate) fn new(window: Duration) -> Self {
        Self {
            window,
            frames: 0,
            elapsed: Duration::ZERO,
        }
    }

    /// Records one frame. Returns the average FPS when a full window has passed, else `None`.
    pub(crate) fn record(&mut self, frame_dt: Duration) -> Option<f32> {
        self.frames += 1;
        self.elapsed += frame_dt;
        if self.elapsed < self.window {
            return None;
        }
        let fps = self.frames as f32 / self.elapsed.as_secs_f32();
        self.frames = 0;
        self.elapsed = Duration::ZERO;
        Some(fps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRAME_60: Duration = Duration::from_nanos(16_666_667);

    #[test]
    fn reports_nothing_before_the_window_is_full() {
        let mut counter = FpsCounter::new(FpsCounter::DEFAULT_WINDOW);
        for _ in 0..29 {
            assert_eq!(counter.record(FRAME_60), None);
        }
    }

    #[test]
    fn reports_the_average_once_the_window_is_full() {
        let mut counter = FpsCounter::new(FpsCounter::DEFAULT_WINDOW);
        let mut reported = None;
        for _ in 0..30 {
            reported = counter.record(FRAME_60);
        }
        let fps = reported.expect("30 frames of 1/60 s fill a 0.5 s window");
        assert!((fps - 60.0).abs() < 0.1, "got {fps}");
    }

    #[test]
    fn starts_a_new_window_after_reporting() {
        let mut counter = FpsCounter::new(Duration::from_secs(1));
        assert_eq!(counter.record(Duration::from_secs(1)), Some(1.0));
        // Ten frames of 0.1 s: the next report is 10 FPS, not influenced by the first window.
        for _ in 0..9 {
            assert_eq!(counter.record(Duration::from_millis(100)), None);
        }
        let fps = counter.record(Duration::from_millis(100)).unwrap();
        assert!((fps - 10.0).abs() < 1e-3, "got {fps}");
    }
}
