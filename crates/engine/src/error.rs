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
}
