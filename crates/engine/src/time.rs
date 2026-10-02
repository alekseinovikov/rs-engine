//! Game time and the fixed timestep.
//!
//! Games advance their simulation in *fixed* steps (here 1/60 s), no matter how fast frames are
//! rendered. A fixed step makes physics deterministic: the same inputs always give the same
//! result, and a fast computer does not make the player jump higher than a slow one.
//!
//! Real frames, however, take a variable amount of time. The classic solution, described in
//! Glenn Fiedler's "Fix Your Timestep!" (<https://gafferongames.com/post/fix_your_timestep/>),
//! is an *accumulator*: every frame adds the real elapsed time to it, and the loop runs as many
//! fixed ticks as fit, keeping the remainder for the next frame:
//!
//! ```text
//! accumulator += frame time (clamped)
//! while accumulator >= fixed_dt { update(); accumulator -= fixed_dt }
//! ```
//!
//! The clamp prevents the "spiral of death": after a long stall (a breakpoint, a dragged window)
//! the loop would otherwise try to catch up with hundreds of ticks, take even longer, and never
//! recover. With the clamp the game simply slows down for a moment instead.
//!
//! [`Time`] does not read a clock. The app measures real time and passes it in as a
//! [`Duration`], which keeps this module pure and easy to test.

use std::time::Duration;

/// Simulation and frame timing, available to the game as `ctx.time`.
#[derive(Debug, Clone)]
pub struct Time {
    fixed_dt: Duration,
    accumulator: Duration,
    frame_dt: Duration,
    ticks: u64,
    fps: f32,
}

// Used by app.rs from Task 7 on; until then only tests call these.
#[allow(dead_code)]
impl Time {
    /// The default fixed step: 1/60 of a second.
    pub const DEFAULT_FIXED_DT: Duration = Duration::from_nanos(16_666_666);

    /// The longest real frame time the accumulator accepts. Longer frames are clamped to it.
    pub const MAX_FRAME_DT: Duration = Duration::from_millis(250);

    /// Creates a clock that steps the simulation by `fixed_dt`.
    ///
    /// # Panics
    ///
    /// Panics if `fixed_dt` is zero: the tick loop would never finish.
    pub fn new(fixed_dt: Duration) -> Self {
        assert!(
            !fixed_dt.is_zero(),
            "fixed_dt must be greater than zero; use Time::DEFAULT_FIXED_DT for 60 Hz"
        );
        Self {
            fixed_dt,
            accumulator: Duration::ZERO,
            frame_dt: Duration::ZERO,
            ticks: 0,
            fps: 0.0,
        }
    }

    /// Adds one real frame's duration to the accumulator, clamped to [`Time::MAX_FRAME_DT`].
    pub(crate) fn advance(&mut self, real_dt: Duration) {
        self.frame_dt = real_dt.min(Self::MAX_FRAME_DT);
        self.accumulator += self.frame_dt;
    }

    /// Takes one fixed step out of the accumulator, if a whole step is available.
    ///
    /// The app calls this in a `while` loop and runs one `Game::update` per `true`.
    pub(crate) fn consume_tick(&mut self) -> bool {
        if self.accumulator < self.fixed_dt {
            return false;
        }
        self.accumulator -= self.fixed_dt;
        self.ticks += 1;
        true
    }

    /// Stores the latest averaged frame rate (measured by the app with an `FpsCounter`).
    pub(crate) fn set_fps(&mut self, fps: f32) {
        self.fps = fps;
    }

    /// The fixed step in seconds. Use it to integrate motion in `Game::update`.
    pub fn fixed_dt(&self) -> f32 {
        self.fixed_dt.as_secs_f32()
    }

    /// The real duration of the current frame in seconds, after clamping.
    pub fn frame_dt(&self) -> f32 {
        self.frame_dt.as_secs_f32()
    }

    /// Simulated time in seconds: the number of ticks so far times the fixed step.
    ///
    /// This is *game* time, not wall-clock time: it stops while the game stalls and is the same
    /// on every computer for the same number of ticks.
    pub fn elapsed(&self) -> f32 {
        // In f64: a `Duration * u32` product would overflow after about two years of ticks.
        (self.fixed_dt.as_secs_f64() * self.ticks as f64) as f32
    }

    /// The number of fixed ticks run so far.
    pub fn ticks(&self) -> u64 {
        self.ticks
    }

    /// Frames per second, averaged over the last half second. Zero until the first measurement.
    pub fn fps(&self) -> f32 {
        self.fps
    }
}

impl Default for Time {
    fn default() -> Self {
        Self::new(Self::DEFAULT_FIXED_DT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const STEP: Duration = Time::DEFAULT_FIXED_DT;

    /// Advances by `real_dt` and returns how many ticks the frame runs.
    fn ticks_for_frame(time: &mut Time, real_dt: Duration) -> u32 {
        time.advance(real_dt);
        let mut ticks = 0;
        while time.consume_tick() {
            ticks += 1;
        }
        ticks
    }

    #[test]
    fn a_frame_of_exactly_one_step_runs_one_tick() {
        let mut time = Time::default();
        assert_eq!(ticks_for_frame(&mut time, STEP), 1);
    }

    #[test]
    fn a_long_frame_runs_several_ticks() {
        let mut time = Time::default();
        assert_eq!(ticks_for_frame(&mut time, STEP * 3), 3);
    }

    #[test]
    fn short_frames_accumulate_until_a_tick_fits() {
        let mut time = Time::default();
        let half = STEP / 2;
        assert_eq!(ticks_for_frame(&mut time, half), 0);
        assert_eq!(ticks_for_frame(&mut time, half), 1);
    }

    #[test]
    fn the_remainder_carries_over_to_the_next_frame() {
        let mut time = Time::default();
        // 1.5 steps: one tick now, half a step left over...
        assert_eq!(ticks_for_frame(&mut time, STEP + STEP / 2), 1);
        // ...which together with another half step makes one more tick.
        assert_eq!(ticks_for_frame(&mut time, STEP / 2), 1);
    }

    #[test]
    fn a_stall_is_clamped_to_the_maximum_frame_time() {
        let mut time = Time::default();
        // 250 ms / 16.666666 ms = 15.0000006: exactly 15 ticks instead of 60 for a 1 s stall.
        assert_eq!(ticks_for_frame(&mut time, Duration::from_secs(1)), 15);
        assert_eq!(time.frame_dt(), Time::MAX_FRAME_DT.as_secs_f32());
    }

    #[test]
    fn elapsed_counts_simulated_time() {
        let mut time = Time::default();
        // Thirty separate frames: a single 0.5 s frame would be clamped to 0.25 s.
        for _ in 0..30 {
            ticks_for_frame(&mut time, STEP);
        }
        assert_eq!(time.ticks(), 30);
        assert!(
            (time.elapsed() - 0.5).abs() < 1e-5,
            "got {}",
            time.elapsed()
        );
    }

    #[test]
    fn fixed_dt_is_in_seconds() {
        let time = Time::default();
        assert!((time.fixed_dt() - 1.0 / 60.0).abs() < 1e-6);
    }

    #[test]
    #[should_panic(expected = "fixed_dt must be greater than zero")]
    fn a_zero_step_is_rejected() {
        Time::new(Duration::ZERO);
    }
}
