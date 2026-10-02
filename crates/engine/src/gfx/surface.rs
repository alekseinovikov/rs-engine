//! The window's surface: the images the GPU draws into and the screen shows.
//!
//! A surface owns a small ring of images, the *swapchain*. Each frame we ask for the next free
//! image (`get_current_texture`), draw into it and hand it back (`present`); the OS shows it at
//! the next screen refresh while we draw the following frame into another image.
//!
//! The surface must be *configured* before use: image size, pixel format and present mode. It
//! must be configured again whenever the window size changes, and it can stop working for
//! reasons outside our control (the window moved to another display, the GPU driver reset). This
//! module handles all of that, so the rest of the renderer only sees "here is an image to draw
//! into" or "skip this frame".

use std::sync::Arc;

use winit::window::Window;

use crate::error::Error;

/// Picks the pixel format: the first sRGB format the surface supports, or `None` without one.
///
/// With an `...Srgb` format, the GPU converts the shader's linear output to sRGB as it writes
/// each pixel (see `color.rs`). `Bgra8UnormSrgb` is guaranteed on every desktop platform; the
/// adapter lists its preferred format first.
fn choose_format(formats: &[wgpu::TextureFormat]) -> Option<wgpu::TextureFormat> {
    formats.iter().copied().find(|format| format.is_srgb())
}

/// The present mode for the vsync setting.
///
/// With vsync, `present` waits for the screen's refresh, so the frame rate matches the display
/// (60, 120 Hz…) and the image never tears. Without it, frames are shown as soon as they are
/// ready. The `Auto…` modes pick the best mode the platform supports and never fail.
fn present_mode(vsync: bool) -> wgpu::PresentMode {
    if vsync {
        wgpu::PresentMode::AutoVsync
    } else {
        wgpu::PresentMode::AutoNoVsync
    }
}

/// What the renderer should do with this frame.
pub(crate) enum Frame {
    /// Draw into this image, then present it.
    Ready(wgpu::SurfaceTexture),
    /// Do not draw this frame; try again on the next one.
    Skip,
}

/// The surface of the game window and its current configuration.
#[derive(Debug)]
pub(crate) struct WindowSurface {
    /// The window, kept so the surface can be recreated if it is lost. The `Arc` lets the
    /// surface hold its own reference, which makes it `Surface<'static>`: no lifetime ties the
    /// renderer to the window's owner.
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    /// Whether the last frame said the surface should be configured again.
    needs_reconfigure: bool,
}

impl WindowSurface {
    /// Creates the surface of `window`. It is configured later, by [`WindowSurface::configure_for`],
    /// once a device exists: the device must be chosen *for* this surface.
    pub(crate) fn create(
        instance: &wgpu::Instance,
        window: Arc<Window>,
    ) -> Result<wgpu::Surface<'static>, Error> {
        Ok(instance.create_surface(window)?)
    }

    /// Configures `surface` for the window's current size.
    pub(crate) fn configure_for(
        surface: wgpu::Surface<'static>,
        window: Arc<Window>,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        vsync: bool,
    ) -> Result<Self, Error> {
        let capabilities = surface.get_capabilities(adapter);
        let format = choose_format(&capabilities.formats).ok_or(Error::UnsupportedSurface)?;
        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            color_space: wgpu::SurfaceColorSpace::Auto,
            width: size.width,
            height: size.height,
            present_mode: present_mode(vsync),
            // Two frames in flight: the CPU prepares one while the GPU draws the other.
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };
        log::info!(
            "surface: {}x{} {format:?}, {:?}",
            config.width,
            config.height,
            config.present_mode
        );
        let surface = Self {
            window,
            surface,
            config,
            needs_reconfigure: false,
        };
        surface.configure(device);
        Ok(surface)
    }

    /// The pixel format of the swapchain images; pipelines must render to the same format.
    pub(crate) fn format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    /// The image size in physical pixels.
    pub(crate) fn size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    /// Whether `present` waits for the screen refresh.
    pub(crate) fn vsync(&self) -> bool {
        self.config.present_mode == wgpu::PresentMode::AutoVsync
    }

    /// Switches vsync on or off.
    pub(crate) fn set_vsync(&mut self, device: &wgpu::Device, vsync: bool) {
        self.config.present_mode = present_mode(vsync);
        self.configure(device);
        log::info!("vsync {}", if vsync { "on" } else { "off" });
    }

    /// Matches the swapchain to a new window size, in physical pixels.
    pub(crate) fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.configure(device);
    }

    /// Gets the image to draw this frame into, repairing the surface if it needs it.
    pub(crate) fn acquire(&mut self, instance: &wgpu::Instance, device: &wgpu::Device) -> Frame {
        if self.needs_reconfigure {
            self.needs_reconfigure = false;
            self.reconfigure_to_window(device);
        }
        // A minimized window has a zero size; wgpu cannot configure (and we cannot draw) a
        // zero-sized surface, so frames are skipped until the window is restored.
        if self.config.width == 0 || self.config.height == 0 {
            return Frame::Skip;
        }
        match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) => Frame::Ready(texture),
            wgpu::CurrentSurfaceTexture::Suboptimal(texture) => {
                // The image still works, but no longer matches the window exactly (for example
                // after moving to a display with a different scale). Draw it, fix it next frame.
                self.needs_reconfigure = true;
                Frame::Ready(texture)
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                // No image became free in time, or the window is hidden: nothing to fix.
                Frame::Skip
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                log::debug!("surface outdated, reconfiguring");
                self.reconfigure_to_window(device);
                Frame::Skip
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                log::warn!("surface lost, recreating it");
                self.recreate(instance, device);
                Frame::Skip
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                log::error!("validation error while acquiring the surface image");
                Frame::Skip
            }
        }
    }

    /// Asks the window to arrange the next frame's timing with the compositor; call it right
    /// before presenting. On Wayland this keeps frames in step with the display.
    pub(crate) fn pre_present(&self) {
        self.window.pre_present_notify();
    }

    /// Configures the surface for the window's size right now. The swapchain can go stale
    /// before the `Resized` event arrives (on Linux especially), and configuring it with the
    /// old size would only make it stale again.
    fn reconfigure_to_window(&mut self, device: &wgpu::Device) {
        let size = self.window.inner_size();
        self.config.width = size.width;
        self.config.height = size.height;
        self.configure(device);
    }

    fn configure(&self, device: &wgpu::Device) {
        if self.config.width > 0 && self.config.height > 0 {
            self.surface.configure(device, &self.config);
        }
    }

    fn recreate(&mut self, instance: &wgpu::Instance, device: &wgpu::Device) {
        match instance.create_surface(Arc::clone(&self.window)) {
            Ok(surface) => {
                self.surface = surface;
                self.configure(device);
            }
            Err(error) => log::error!("failed to recreate the surface: {error}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wgpu::TextureFormat;

    #[test]
    fn the_first_srgb_format_wins() {
        let formats = [
            TextureFormat::Bgra8Unorm,
            TextureFormat::Bgra8UnormSrgb,
            TextureFormat::Rgba8UnormSrgb,
        ];
        assert_eq!(choose_format(&formats), Some(TextureFormat::Bgra8UnormSrgb));
    }

    #[test]
    fn a_surface_without_srgb_formats_is_unsupported() {
        let formats = [TextureFormat::Bgra8Unorm, TextureFormat::Rgba16Float];
        assert_eq!(choose_format(&formats), None);
    }

    #[test]
    fn vsync_selects_an_auto_present_mode() {
        assert_eq!(present_mode(true), wgpu::PresentMode::AutoVsync);
        assert_eq!(present_mode(false), wgpu::PresentMode::AutoNoVsync);
    }
}
