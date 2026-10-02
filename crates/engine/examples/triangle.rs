//! M2 example: the first pixels from the GPU.
//!
//! Run it with `cargo run -p engine --example triangle`. It shows a triangle with a red, a
//! green and a blue corner (the GPU blends the colors across it) and a solid quad drawn with an
//! index buffer, on a cleared background. Then:
//!
//! - press C: cycles the background color;
//! - press V: toggles vsync. With vsync the FPS in the title matches the display's refresh rate;
//!   without it, it jumps to hundreds or thousands;
//! - press F: toggles a 30 FPS frame limit, which works with and without vsync;
//! - resize the window: the shapes stretch with it, because clip space always spans the window;
//! - press Esc or close the window to quit.

use engine::{Color, Config, Context, Game, Key, Vertex};

const FRAME_LIMIT: u32 = 30;

/// Background colors to cycle through, as `0xRRGGBB` like in an image editor.
const BACKGROUNDS: [u32; 4] = [0x1E1E2E, 0x2E3440, 0x0B3D2E, 0xF5F0E1];

struct TriangleDemo {
    background: usize,
}

impl Game for TriangleDemo {
    fn update(&mut self, ctx: &mut Context) {
        if ctx.input.just_pressed(Key::C) {
            self.background = (self.background + 1) % BACKGROUNDS.len();
            let hex = BACKGROUNDS[self.background];
            ctx.gfx.set_clear_color(Color::from_hex(hex));
            log::info!("background #{hex:06X}");
        }
        if ctx.input.just_pressed(Key::V) {
            let vsync = !ctx.gfx.vsync();
            ctx.gfx.set_vsync(vsync);
        }
        if ctx.input.just_pressed(Key::F) {
            let limit = match ctx.max_fps() {
                Some(_) => None,
                None => Some(FRAME_LIMIT),
            };
            ctx.set_max_fps(limit);
            log::info!("frame limit: {limit:?}");
        }
    }

    fn draw(&mut self, ctx: &mut Context) {
        // Clip space: x and y from -1 to 1, +y up, the window's center at (0, 0).
        ctx.gfx.draw_triangle([
            Vertex::new([-0.35, 0.6], Color::RED),
            Vertex::new([-0.85, -0.5], Color::GREEN),
            Vertex::new([0.15, -0.5], Color::BLUE),
        ]);
        ctx.gfx
            .draw_quad([0.3, -0.5], [0.8, 0.3], Color::from_hex(0xF2C14E));
        // A half-transparent quad over both shapes shows alpha blending.
        ctx.gfx
            .draw_quad([-0.2, -0.7], [0.5, -0.2], Color::WHITE.with_alpha(0.5));
    }
}

fn main() -> Result<(), engine::Error> {
    // Show `info` and above unless RUST_LOG says otherwise.
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let config = Config {
        title: "rs-engine: triangle".to_string(),
        ..Default::default()
    };
    engine::run(config, |ctx| {
        ctx.gfx.set_clear_color(Color::from_hex(BACKGROUNDS[0]));
        TriangleDemo { background: 0 }
    })
}
