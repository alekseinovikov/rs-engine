//! Graphics: everything between the game's draw calls and the pixels on screen.
//!
//! The renderer is *immediate-mode*: during [`Game::draw`](crate::Game::draw) the game describes
//! the whole frame again (`ctx.gfx.draw_triangle(...)`, `ctx.gfx.draw_quad(...)`), and nothing
//! is remembered from the previous frame. The engine wraps every draw like this:
//!
//! ```text
//! gfx.begin_frame()     forget last frame's shapes
//! game.draw(ctx)        the game submits shapes; they are collected on the CPU
//! gfx.end_frame()       acquire a swapchain image, upload the shapes, record a render pass
//!                       that clears the image and draws them, submit it, present the image
//! ```
//!
//! The pieces, one file each:
//!
//! - `color`: [`Color`], sRGB colors and their conversion to linear;
//! - `gpu`: the instance, adapter, device and queue;
//! - `surface`: the window's swapchain, resizing, vsync and recovery from a lost surface;
//! - `shapes`: [`Vertex`] and the CPU-side batch of colored triangles;
//! - `buffer`: GPU buffers that grow to fit the frame;
//! - `shape_pipeline` and `shapes.wgsl`: the shaders and the pipeline that draw the batch.

mod buffer;
mod color;
mod gpu;
mod shape_pipeline;
mod shapes;
mod surface;

use std::fmt;
use std::sync::Arc;

use winit::window::Window;

pub use color::Color;
pub use shapes::Vertex;

use crate::error::Error;
use gpu::Gpu;
use shape_pipeline::ShapePipeline;
use shapes::ShapeBatch;
use surface::{Frame, WindowSurface};

/// The renderer, reached by the game as `ctx.gfx`.
pub struct Gfx {
    instance: wgpu::Instance,
    gpu: Gpu,
    surface: WindowSurface,
    shapes: ShapePipeline,
    batch: ShapeBatch,
    clear_color: Color,
}

impl Gfx {
    /// The clear color before the game sets its own: a dark blue-grey.
    pub const DEFAULT_CLEAR_COLOR: Color = Color::from_hex(0x1E1E2E);

    /// Connects to the GPU and sets up drawing into `window`.
    pub(crate) fn new(window: Arc<Window>, vsync: bool) -> Result<Self, Error> {
        // The display handle lets backends that need one (OpenGL on Linux, for example) talk to
        // the display server. `from_env` lets `WGPU_BACKEND=vulkan` and friends override the
        // defaults without recompiling. In debug builds the default flags turn on validation:
        // every API call is checked, and a mistake panics with a readable message.
        let descriptor = wgpu::InstanceDescriptor::new_with_display_handle_from_env(Box::new(
            Arc::clone(&window),
        ));
        let instance = wgpu::Instance::new(descriptor);
        // The surface comes first: the adapter must be one that can draw to it.
        let raw_surface = WindowSurface::create(&instance, Arc::clone(&window))?;
        let gpu = Gpu::new(&instance, &raw_surface)?;
        let surface =
            WindowSurface::configure_for(raw_surface, window, &gpu.adapter, &gpu.device, vsync)?;
        let shapes = ShapePipeline::new(&gpu.device, surface.format());
        Ok(Self {
            instance,
            gpu,
            surface,
            shapes,
            batch: ShapeBatch::default(),
            clear_color: Self::DEFAULT_CLEAR_COLOR,
        })
    }

    /// The color the screen is cleared to at the start of every frame.
    pub fn clear_color(&self) -> Color {
        self.clear_color
    }

    /// Sets the color the screen is cleared to, from this frame on.
    pub fn set_clear_color(&mut self, color: Color) {
        self.clear_color = color;
    }

    /// Draws a triangle with a color per corner; the GPU blends the colors across it.
    ///
    /// Corners are in clip space (see [`Vertex`]). Temporary M2 API: M3 replaces it with
    /// world-space drawing through a camera.
    pub fn draw_triangle(&mut self, corners: [Vertex; 3]) {
        self.batch.push_triangle(corners);
    }

    /// Draws a solid rectangle from `min` (bottom left) to `max` (top right), in clip space.
    ///
    /// Temporary M2 API: M3 replaces it with `draw_rect` in world pixels.
    pub fn draw_quad(&mut self, min: [f32; 2], max: [f32; 2], color: Color) {
        self.batch.push_quad(min, max, color);
    }

    /// Whether presenting waits for the screen refresh.
    pub fn vsync(&self) -> bool {
        self.surface.vsync()
    }

    /// Turns vsync on or off. Takes effect from the next frame.
    pub fn set_vsync(&mut self, vsync: bool) {
        self.surface.set_vsync(&self.gpu.device, vsync);
    }

    /// The size of the drawing area in physical pixels (on a Retina display, twice the window's
    /// logical size).
    pub fn surface_size(&self) -> (u32, u32) {
        self.surface.size()
    }

    /// Matches the swapchain to a new window size, in physical pixels.
    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        self.surface.resize(&self.gpu.device, width, height);
    }

    /// Starts a frame: forgets the previous frame's shapes.
    pub(crate) fn begin_frame(&mut self) {
        self.batch.clear();
    }

    /// Finishes the frame: draws everything submitted since [`Gfx::begin_frame`] and shows it.
    pub(crate) fn end_frame(&mut self) {
        let Frame::Ready(texture) = self.surface.acquire(&self.instance, &self.gpu.device) else {
            return;
        };
        let device = &self.gpu.device;
        let queue = &self.gpu.queue;
        self.shapes.upload(device, queue, &self.batch);

        // A view says how to interpret a texture (which format, which mip levels); the default
        // view of a swapchain image is the whole image in its own format.
        let view = texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        // GPU work is not executed call by call: we *record* commands into an encoder, finish it
        // into a command buffer and submit that buffer to the queue in one go.
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("frame encoder"),
        });
        {
            // A render pass draws into a set of images (here one, the swapchain image). `Clear`
            // fills it with the clear color first; `Store` keeps the result for presenting.
            let [r, g, b, a] = self.clear_color.to_linear();
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: f64::from(r),
                            g: f64::from(g),
                            b: f64::from(b),
                            a: f64::from(a),
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            self.shapes.draw(&mut pass, &self.batch);
            // The pass borrows the encoder; it must end (be dropped) before `finish`.
        }
        queue.submit([encoder.finish()]);
        self.surface.pre_present();
        queue.present(texture);
    }
}

impl fmt::Debug for Gfx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Gfx")
            .field("surface_size", &self.surface_size())
            .field("vsync", &self.vsync())
            .field("clear_color", &self.clear_color)
            .finish_non_exhaustive()
    }
}
