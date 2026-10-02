//! M1 example: a window driven by the fixed-timestep game loop.
//!
//! Run it with `cargo run -p engine --example window`, then:
//!
//! - press keys and click: every press and release is logged;
//! - press Space: toggles a once-per-second report of how many `update` ticks ran;
//! - press F: toggles the 60 FPS frame limit. The FPS in the title jumps from 60 to tens of
//!   thousands, while the tick report stays at 60: the simulation does not depend on the frame
//!   rate;
//! - press Esc or close the window to quit.

use std::time::{Duration, Instant};

use engine::{Config, Context, Game, Key, MouseButton};

const FRAME_LIMIT: u32 = 60;

struct WindowDemo {
    report_ticks: bool,
    ticks_this_second: u32,
    second_started: Instant,
}

impl WindowDemo {
    fn new() -> Self {
        Self {
            report_ticks: false,
            ticks_this_second: 0,
            second_started: Instant::now(),
        }
    }

    fn log_input(ctx: &Context) {
        for key in ctx.input.just_pressed_keys() {
            log::info!("pressed {key:?}");
        }
        for key in ctx.input.just_released_keys() {
            log::info!("released {key:?}");
        }
        for button in [MouseButton::Left, MouseButton::Right, MouseButton::Middle] {
            if ctx.input.mouse_just_pressed(button) {
                let (x, y) = ctx.input.mouse_position();
                log::info!("{button:?} click at ({x:.0}, {y:.0})");
            }
        }
    }

    /// Counts ticks against the wall clock, which the engine's `Time` deliberately does not use.
    fn count_ticks(&mut self, ctx: &Context) {
        self.ticks_this_second += 1;
        let elapsed = self.second_started.elapsed();
        if elapsed >= Duration::from_secs(1) {
            if self.report_ticks {
                log::info!(
                    "{} ticks in {:.2} s, {:.0} FPS",
                    self.ticks_this_second,
                    elapsed.as_secs_f32(),
                    ctx.time.fps()
                );
            }
            self.ticks_this_second = 0;
            self.second_started = Instant::now();
        }
    }
}

impl Game for WindowDemo {
    fn update(&mut self, ctx: &mut Context) {
        Self::log_input(ctx);

        if ctx.input.just_pressed(Key::Space) {
            self.report_ticks = !self.report_ticks;
            log::info!(
                "tick report {}",
                if self.report_ticks { "on" } else { "off" }
            );
        }
        if ctx.input.just_pressed(Key::F) {
            let limit = match ctx.max_fps() {
                Some(_) => None,
                None => Some(FRAME_LIMIT),
            };
            ctx.set_max_fps(limit);
            log::info!("frame limit: {limit:?}");
        }

        self.count_ticks(ctx);
    }

    fn draw(&mut self, _ctx: &mut Context) {
        // Nothing to draw: the engine clears the screen; see the triangle example for shapes.
    }
}

fn main() -> Result<(), engine::Error> {
    // Show `info` and above unless RUST_LOG says otherwise.
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let config = Config {
        title: "rs-engine: window".to_string(),
        max_fps: Some(FRAME_LIMIT),
        ..Default::default()
    };
    engine::run(config, |_ctx| WindowDemo::new())
}
