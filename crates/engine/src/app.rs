//! The glue between winit and the game: the window, OS events and the frame.
//!
//! winit 0.30 drives the program through the `ApplicationHandler` trait. Instead of us calling
//! "get next event" in a loop, winit owns the loop and calls our methods:
//!
//! - `resumed`: the app may create windows now (on some platforms windows cannot exist
//!   earlier), so this is where the window and the game are created;
//! - `window_event`: one OS event (a key, the mouse, the close button, a redraw request);
//! - `about_to_wait`: all pending events are handled and winit is about to sleep. Here we decide
//!   when the next frame runs: right away (`ControlFlow::Poll`) or at the frame limiter's
//!   deadline (`ControlFlow::WaitUntil`).
//!
//! A frame runs on `RedrawRequested`, which we request ourselves from `about_to_wait`. That is
//! also the event the OS sends when the window must be repainted (after a resize, for example),
//! so in M2 rendering will naturally live here.

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
use crate::input::Key;
use crate::pacing::FramePacer;
use crate::platform;

/// The window and the game, which exist only after `resumed`.
struct Running<G> {
    window: Window,
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
    ctx: Context,
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
        let ctx = Context::new(&config);
        let pacer = FramePacer::new(config.max_fps);
        Self {
            config,
            make_game: Some(make_game),
            running: None,
            ctx,
            pacer,
            fps: FpsCounter::new(FpsCounter::DEFAULT_WINDOW),
            error: None,
        }
    }

    /// Takes the error stored during the run, if any.
    pub(crate) fn take_error(&mut self) -> Option<Error> {
        self.error.take()
    }

    /// Runs one frame: as many fixed ticks as the accumulated time allows, then one draw.
    fn frame(&mut self, event_loop: &ActiveEventLoop) {
        let Some(running) = &mut self.running else {
            return;
        };

        let now = Instant::now();
        let frame_dt = now - running.last_frame;
        running.last_frame = now;
        self.pacer.frame_started(now);

        self.ctx.time.advance(frame_dt);
        while self.ctx.time.consume_tick() {
            running.game.update(&mut self.ctx);
            // Edges live for exactly one tick.
            self.ctx.input.end_tick();
        }
        running.game.draw(&mut self.ctx);

        if let Some(fps) = self.fps.record(frame_dt) {
            self.ctx.time.set_fps(fps);
            let title = format!("{} — {fps:.0} FPS", self.config.title);
            running.window.set_title(&title);
        }

        // The game may have changed the limit or asked to quit during this frame.
        self.pacer.set_max_fps(self.ctx.max_fps());
        if self.ctx.quit_requested() {
            event_loop.exit();
        }
    }

    fn handle_key(&mut self, event_loop: &ActiveEventLoop, code: PhysicalKey, state: ElementState) {
        let PhysicalKey::Code(code) = code else {
            return;
        };
        let Some(key) = platform::key_from_winit(code) else {
            return;
        };
        match state {
            ElementState::Pressed => {
                self.ctx.input.press_key(key);
                if key == Key::Escape && self.config.quit_on_escape {
                    log::info!("Esc pressed, quitting");
                    event_loop.exit();
                }
            }
            ElementState::Released => self.ctx.input.release_key(key),
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
        let attributes = Window::default_attributes()
            .with_title(self.config.title.clone())
            .with_inner_size(LogicalSize::new(self.config.width, self.config.height));
        let window = match event_loop.create_window(attributes) {
            Ok(window) => window,
            Err(error) => {
                self.error = Some(Error::Window(error));
                event_loop.exit();
                return;
            }
        };
        log::info!(
            "window created: {}x{} logical pixels, frame limit {:?}",
            self.config.width,
            self.config.height,
            self.config.max_fps
        );

        let make_game = self
            .make_game
            .take()
            .expect("the game is created only once, on the first `resumed`");
        let game = make_game(&mut self.ctx);
        self.running = Some(Running {
            window,
            game,
            last_frame: Instant::now(),
        });
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => self.frame(event_loop),
            WindowEvent::KeyboardInput { event, .. } => {
                self.handle_key(event_loop, event.physical_key, event.state);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if let Some(button) = platform::mouse_button_from_winit(button) {
                    match state {
                        ElementState::Pressed => self.ctx.input.press_mouse(button),
                        ElementState::Released => self.ctx.input.release_mouse(button),
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.ctx
                    .input
                    .set_mouse_position(position.x as f32, position.y as f32);
            }
            // Key-ups that happen in another window never reach us; without this, a key held
            // while switching windows would stay "down" forever.
            WindowEvent::Focused(false) => self.ctx.input.release_all(),
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
