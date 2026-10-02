//! Graphics: everything between the game's draw calls and the pixels on screen.

// Used by the renderer from Task 6 on.
#[allow(dead_code)]
mod buffer;
mod color;
// Used by the renderer from Task 6 on.
#[allow(dead_code)]
mod gpu;
// The renderer that uses the batch arrives in Task 6; until then only the tests do.
#[allow(dead_code)]
mod shapes;
#[allow(dead_code)]
mod surface;

pub use color::Color;
pub use shapes::Vertex;
