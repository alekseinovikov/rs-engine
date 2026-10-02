//! The connection to the GPU: instance, adapter, device and queue.
//!
//! wgpu mirrors the WebGPU standard, which splits "talking to the GPU" into four objects:
//!
//! - the **instance** is the entry point: it loads the platform's graphics API (Metal on macOS,
//!   Vulkan on Linux, DX12 on Windows) and creates surfaces;
//! - an **adapter** is one physical GPU (or a software renderer) and tells us what it supports;
//! - the **device** is our logical, private connection to that adapter: every buffer, texture,
//!   shader and pipeline is created through it;
//! - the **queue** is where finished command buffers go to be executed, and where small uploads
//!   (`write_buffer`) are scheduled.
//!
//! The adapter and device are requested with `async` functions, because in a browser they
//! really are asynchronous. On desktop they finish immediately, so we block on them with
//! `pollster` instead of pulling in an async runtime.

use crate::error::Error;

/// The device and queue, plus the adapter they came from.
#[derive(Debug)]
pub(crate) struct Gpu {
    pub(crate) adapter: wgpu::Adapter,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
}

impl Gpu {
    /// Picks a GPU that can draw to `surface` and opens a device on it.
    pub(crate) fn new(instance: &wgpu::Instance, surface: &wgpu::Surface) -> Result<Self, Error> {
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            // Prefer the discrete GPU on laptops that have two; a 2D game barely needs it, but
            // it avoids surprises when M3 draws 10 000 sprites.
            power_preference: wgpu::PowerPreference::HighPerformance,
            // Only adapters that can present to our window are useful.
            compatible_surface: Some(surface),
            ..Default::default()
        }))?;
        let info = adapter.get_info();
        log::info!("GPU: {} ({:?})", info.name, info.backend);

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                label: Some("rs-engine device"),
                // The most conservative limits: what every GPU, even old ones and WebGL, supports.
                // A 2D engine stays far below them, and asking for little means the device
                // request cannot fail because of limits.
                required_limits:
                    wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
                ..Default::default()
            }))?;

        Ok(Self {
            adapter,
            device,
            queue,
        })
    }
}
