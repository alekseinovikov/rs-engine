//! The glue between winit and the game: the window, OS events and the frame.
//!
//! winit 0.30 drives the program through the `ApplicationHandler` trait. Instead of us calling
//! "get next event" in a loop, winit owns the loop and calls our methods:
//!
//! - `resumed`: the app may create windows now (on some platforms windows cannot exist
//!   earlier), so this is where the window, the renderer, the context and the game are created;
//! - `window_event`: one OS event (a key, the mouse, a resize, the close button, a redraw
//!   request);
//! - `about_to_wait`: all pending events are handled and winit is about to sleep. Here we decide
//!   when the next frame runs: right away (`ControlFlow::Poll`) or at the frame limiter's
//!   deadline (`ControlFlow::WaitUntil`).
//!
//! A frame runs on `RedrawRequested`, which we request ourselves from `about_to_wait`. That is
//! also the event the OS sends when the window must be repainted (after a resize, for example),
//! so the window never shows stale content.

use std::sync::Arc;
use std::time::Instant;

use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowId};

use crate::Game;
use crate::config::Config;
use crate::context::Context;
use crate::error::Error;
use crate::fps::FpsCounter;
use crate::gfx::Gfx;
use crate::input::Key;
use crate::pacing::FramePacer;
use crate::platform;

/// Everything that exists only after `resumed`: the window, the context (which owns the
/// renderer) and the game.
struct Running<G> {
    /// Shared with the renderer's surface, which needs the window to outlive it.
    window: Arc<Window>,
    ctx: Context,
    game: G,
    last_frame: Instant,
}

/// The engine's state while the event loop runs.
pub(crate) struct App<G, F> {
    config: Config,
    /// The game's constructor. It is an `Option` because it is called once, in `resumed`, and
    /// `FnOnce` must be moved out to be called.
    make_game: Option<F>,
    running: Option<Running<G>>,
    pacer: FramePacer,
    fps: FpsCounter,
    /// An error from inside a callback. Callbacks cannot return a `Result`, so we store the
    /// error, stop the loop, and `run` returns it afterwards.
    error: Option<Error>,
}

impl<G, F> App<G, F>
where
    G: Game,
    F: FnOnce(&mut Context) -> G,
{
    pub(crate) fn new(config: Config, make_game: F) -> Self {
        let pacer = FramePacer::new(config.max_fps);
        Self {
            config,
            make_game: Some(make_game),
            running: None,
            pacer,
            fps: FpsCounter::new(FpsCounter::DEFAULT_WINDOW),
            error: None,
        }
    }

    /// Takes the error stored during the run, if any.
    pub(crate) fn take_error(&mut self) -> Option<Error> {
        self.error.take()
    }

    /// Creates the window, the renderer, the context and the game.
    fn start(&mut self, event_loop: &ActiveEventLoop) -> Result<Running<G>, Error> {
        let attributes = Window::default_attributes()
            .with_title(self.config.title.clone())
            .with_inner_size(LogicalSize::new(self.config.width, self.config.height));
        let window = Arc::new(event_loop.create_window(attributes)?);
        log::info!(
            "window created: {}x{} logical pixels, frame limit {:?}",
            self.config.width,
            self.config.height,
            self.config.max_fps
        );

        let gfx = Gfx::new(Arc::clone(&window), self.config.vsync)?;
        let mut ctx = Context::new(&self.config, gfx);
        let make_game = self
            .make_game
            .take()
            .expect("the game is created only once, on the first `resumed`");
        let game = make_game(&mut ctx);
        Ok(Running {
            window,
            ctx,
            game,
            last_frame: Instant::now(),
        })
    }

    /// Runs one frame: as many fixed ticks as the accumulated time allows, then one draw.
    fn frame(&mut self, event_loop: &ActiveEventLoop) {
        let Some(running) = &mut self.running else {
            return;
        };
        let ctx = &mut running.ctx;

        let now = Instant::now();
        let frame_dt = now - running.last_frame;
        running.last_frame = now;
        self.pacer.frame_started(now);

        ctx.time.advance(frame_dt);
        while ctx.time.consume_tick() {
            running.game.update(ctx);
            // Edges live for exactly one tick.
            ctx.input.end_tick();
        }
        ctx.gfx.begin_frame();
        running.game.draw(ctx);
        ctx.gfx.end_frame();

        if let Some(fps) = self.fps.record(frame_dt) {
            ctx.time.set_fps(fps);
            let title = format!("{} — {fps:.0} FPS", self.config.title);
            running.window.set_title(&title);
        }

        // The game may have changed the limit or asked to quit during this frame.
        self.pacer.set_max_fps(ctx.max_fps());
        if ctx.quit_requested() {
            event_loop.exit();
        }
    }
}

/// Feeds one key event into the input state; returns whether it should quit the engine.
fn handle_key(ctx: &mut Context, config: &Config, code: PhysicalKey, state: ElementState) -> bool {
    let PhysicalKey::Code(code) = code else {
        return false;
    };
    let Some(key) = platform::key_from_winit(code) else {
        return false;
    };
    match state {
        ElementState::Pressed => {
            ctx.input.press_key(key);
            key == Key::Escape && config.quit_on_escape
        }
        ElementState::Released => {
            ctx.input.release_key(key);
            false
        }
    }
}

impl<G, F> ApplicationHandler for App<G, F>
where
    G: Game,
    F: FnOnce(&mut Context) -> G,
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // On desktop `resumed` comes once; on mobile it repeats after every suspend.
        if self.running.is_some() {
            return;
        }
        match self.start(event_loop) {
            Ok(running) => self.running = Some(running),
            Err(error) => {
                self.error = Some(error);
                event_loop.exit();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let WindowEvent::RedrawRequested = event {
            self.frame(event_loop);
            return;
        }
        // Events can only arrive once the window exists, so `running` is always `Some` here.
        let Some(running) = &mut self.running else {
            return;
        };
        let ctx = &mut running.ctx;
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            // The new size is in physical pixels, exactly what the swapchain needs.
            WindowEvent::Resized(size) => ctx.gfx.resize(size.width, size.height),
            WindowEvent::KeyboardInput { event, .. } => {
                if handle_key(ctx, &self.config, event.physical_key, event.state) {
                    log::info!("Esc pressed, quitting");
                    event_loop.exit();
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if let Some(button) = platform::mouse_button_from_winit(button) {
                    match state {
                        ElementState::Pressed => ctx.input.press_mouse(button),
                        ElementState::Released => ctx.input.release_mouse(button),
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                ctx.input
                    .set_mouse_position(position.x as f32, position.y as f32);
            }
            // Key-ups that happen in another window never reach us; without this, a key held
            // while switching windows would stay "down" forever.
            WindowEvent::Focused(false) => ctx.input.release_all(),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let Some(running) = &self.running else {
            return;
        };
        match self.pacer.next_frame() {
            Some(deadline) if Instant::now() < deadline => {
                event_loop.set_control_flow(ControlFlow::WaitUntil(deadline));
            }
            _ => {
                running.window.request_redraw();
                event_loop.set_control_flow(ControlFlow::Poll);
            }
        }
    }
}
