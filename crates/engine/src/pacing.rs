//! The frame limiter: when should the next frame start?
//!
//! Without vsync nothing slows the loop down, and an empty frame takes a few
//! microseconds, so the loop would spin at tens of thousands of frames per second and keep a CPU
//! core at 100 %. With a limit of `n` frames per second, [`FramePacer`] computes the instant the
//! next frame is due, and the app asks winit to sleep until then (`ControlFlow::WaitUntil`).
//!
//! Deadlines are spaced exactly one period apart (`next = previous deadline + period`) rather
//! than measured from when a frame actually started (`next = now + period`). The OS wakes us a
//! little late every time; measuring from `now` would add that lateness to every frame and give,
//! say, 58 FPS instead of 60. If we fall behind by more than a whole period (a stall), the
//! schedule restarts from `now` instead of trying to catch up with a burst of frames.
//!
//! The pacer also backs off when a frame could not be shown at all, for example while the window
//! is minimized or hidden behind another one: the GPU then reports at once that there is nothing
//! to draw into, and without a pause the loop would spin a core at 100 % until the window is back.

use std::time::{Duration, Instant};

/// Converts a frame limit into the time between frames.
///
/// # Panics
///
/// Panics on `Some(0)`: zero frames per second is not a limit. Use `None` for "no limit".
pub(crate) fn frame_period(max_fps: Option<u32>) -> Option<Duration> {
    max_fps.map(|fps| {
        assert!(
            fps > 0,
            "max_fps must be at least 1; use None to disable the limit"
        );
        Duration::from_secs(1) / fps
    })
}

/// How long to wait after a frame that could not be shown before trying again. Short enough that
/// the game reappears without a visible delay, long enough that a hidden window costs nothing.
pub(crate) const SKIPPED_FRAME_RETRY: Duration = Duration::from_millis(100);

/// Schedules frames under an optional frames-per-second limit.
#[derive(Debug, Clone)]
pub(crate) struct FramePacer {
    max_fps: Option<u32>,
    period: Option<Duration>,
    next_frame: Option<Instant>,
}

impl FramePacer {
    /// Creates a pacer. `None` means frames run back to back.
    pub(crate) fn new(max_fps: Option<u32>) -> Self {
        Self {
            max_fps,
            period: frame_period(max_fps),
            next_frame: None,
        }
    }

    /// Changes the limit. The new limit applies from the next frame on.
    pub(crate) fn set_max_fps(&mut self, max_fps: Option<u32>) {
        if max_fps == self.max_fps {
            return;
        }
        self.max_fps = max_fps;
        self.period = frame_period(max_fps);
        // Restart the schedule: the next frame runs immediately and sets up the new grid.
        self.next_frame = None;
    }

    /// Tells the pacer that a frame starts at `now`, which schedules the next one.
    pub(crate) fn frame_started(&mut self, now: Instant) {
        let Some(period) = self.period else {
            self.next_frame = None;
            return;
        };
        let next = match self.next_frame {
            // Stay on the grid while we keep up with it...
            Some(previous) if previous + period > now => previous + period,
            // ...but after the first frame or a stall, start a new grid from now.
            _ => now + period,
        };
        self.next_frame = Some(next);
    }

    /// Tells the pacer that the frame started at `now` could not be shown, which delays the next
    /// one by at least [`SKIPPED_FRAME_RETRY`]. Call it after [`FramePacer::frame_started`].
    pub(crate) fn frame_skipped(&mut self, now: Instant) {
        let retry = now + SKIPPED_FRAME_RETRY;
        self.next_frame = Some(match self.next_frame {
            Some(next) => next.max(retry),
            None => retry,
        });
    }

    /// The instant the next frame is due, or `None` if it is due immediately.
    pub(crate) fn next_frame(&self) -> Option<Instant> {
        self.next_frame
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MS: Duration = Duration::from_millis(1);

    #[test]
    fn frame_period_converts_fps_to_a_duration() {
        assert_eq!(frame_period(None), None);
        assert_eq!(frame_period(Some(50)), Some(Duration::from_millis(20)));
    }

    #[test]
    #[should_panic(expected = "max_fps must be at least 1")]
    fn frame_period_rejects_zero() {
        frame_period(Some(0));
    }

    #[test]
    fn without_a_limit_every_frame_is_due_immediately() {
        let mut pacer = FramePacer::new(None);
        pacer.frame_started(Instant::now());
        assert_eq!(pacer.next_frame(), None);
    }

    #[test]
    fn the_first_frame_schedules_the_next_one_period_later() {
        let start = Instant::now();
        let mut pacer = FramePacer::new(Some(50));
        pacer.frame_started(start);
        assert_eq!(pacer.next_frame(), Some(start + 20 * MS));
    }

    #[test]
    fn a_late_wake_up_does_not_shift_the_schedule() {
        let start = Instant::now();
        let mut pacer = FramePacer::new(Some(50));
        pacer.frame_started(start);
        // The OS wakes us 2 ms late; the next deadline still stays on the 20 ms grid.
        pacer.frame_started(start + 22 * MS);
        assert_eq!(pacer.next_frame(), Some(start + 40 * MS));
    }

    #[test]
    fn after_a_stall_the_schedule_restarts_from_now() {
        let start = Instant::now();
        let mut pacer = FramePacer::new(Some(50));
        pacer.frame_started(start);
        let late = start + 100 * MS;
        pacer.frame_started(late);
        assert_eq!(pacer.next_frame(), Some(late + 20 * MS));
    }

    #[test]
    fn removing_the_limit_makes_frames_due_immediately() {
        let start = Instant::now();
        let mut pacer = FramePacer::new(Some(50));
        pacer.frame_started(start);
        pacer.set_max_fps(None);
        assert_eq!(pacer.next_frame(), None);
    }

    #[test]
    fn setting_the_same_limit_keeps_the_schedule() {
        let start = Instant::now();
        let mut pacer = FramePacer::new(Some(50));
        pacer.frame_started(start);
        pacer.set_max_fps(Some(50));
        assert_eq!(pacer.next_frame(), Some(start + 20 * MS));
    }

    #[test]
    fn a_skipped_frame_delays_the_next_one_without_a_limit() {
        let now = Instant::now();
        let mut pacer = FramePacer::new(None);
        pacer.frame_started(now);
        pacer.frame_skipped(now);
        assert_eq!(pacer.next_frame(), Some(now + SKIPPED_FRAME_RETRY));

        // A frame that is shown again runs the loop at full speed.
        let later = now + SKIPPED_FRAME_RETRY;
        pacer.frame_started(later);
        assert_eq!(pacer.next_frame(), None);
    }

    #[test]
    fn a_skipped_frame_waits_for_whichever_is_later() {
        let now = Instant::now();
        let mut fast = FramePacer::new(Some(60));
        fast.frame_started(now);
        fast.frame_skipped(now);
        assert_eq!(fast.next_frame(), Some(now + SKIPPED_FRAME_RETRY));

        let mut slow = FramePacer::new(Some(5));
        slow.frame_started(now);
        slow.frame_skipped(now);
        assert_eq!(slow.next_frame(), Some(now + 200 * MS));
    }
}
