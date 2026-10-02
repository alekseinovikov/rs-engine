//! The engine's error type.
//!
//! Fallible engine functions return `Result<T, engine::Error>`. Each variant wraps the error of
//! the library that failed, so `{}` prints our context and `std::error::Error::source` gives the
//! original cause. `thiserror` writes the `Display` and `Error` impls from the attributes.

/// Everything that can go wrong inside the engine.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The OS event loop could not be created or failed while running.
    #[error("failed to run the event loop: {0}")]
    EventLoop(#[from] winit::error::EventLoopError),
    /// The window could not be created.
    #[error("failed to create the window: {0}")]
    Window(#[from] winit::error::OsError),
    /// The GPU surface for the window could not be created.
    #[error("failed to create the GPU surface: {0}")]
    CreateSurface(#[from] wgpu::CreateSurfaceError),
    /// No GPU (or software renderer) can draw to the window.
    #[error("no suitable GPU found: {0}")]
    RequestAdapter(#[from] wgpu::RequestAdapterError),
    /// The GPU was found but refused to open a device.
    #[error("failed to open the GPU device: {0}")]
    RequestDevice(#[from] wgpu::RequestDeviceError),
    /// The window's surface offers no sRGB pixel format.
    #[error("the window surface supports no sRGB format")]
    UnsupportedSurface,
}
