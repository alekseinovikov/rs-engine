//! Colored shapes collected on the CPU during a frame, ready to be uploaded to the GPU.
//!
//! A GPU does not draw "a triangle" or "a rectangle"; it draws a list of *vertices* (corner
//! points) and an *index* list that says which three vertices form each triangle. A rectangle is
//! two triangles that share two corners, so with indices it needs 4 vertices instead of 6:
//!
//! ```text
//!  0 ──── 1      triangle A: 0, 2, 1
//!  │ A  ╱ │      triangle B: 1, 2, 3
//!  │  ╱ B │
//!  2 ──── 3
//! ```
//!
//! [`ShapeBatch`] gathers every shape of the frame into one vertex list and one index list, so
//! the whole frame is a single draw call. It is plain Rust with no GPU types, so it is tested
//! directly. Its two `Vec`s are cleared, not freed, between frames: after the first few frames
//! no memory is allocated at all.

use bytemuck::{Pod, Zeroable};

use super::color::Color;

/// A corner of a shape: a position and the color at that corner.
///
/// Positions are in *clip space*, the GPU's own coordinates: `x` from −1 (left edge) to 1
/// (right edge), `y` from −1 (bottom) to 1 (top). This is a temporary M2 API: from M3 on, games
/// draw in world pixels through a camera.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vertex {
    /// The position in clip space.
    pub position: [f32; 2],
    /// The color at this corner, in sRGB. The GPU blends the corner colors across the shape.
    pub color: Color,
}

impl Vertex {
    /// A vertex at `position` (clip space) with `color`.
    pub const fn new(position: [f32; 2], color: Color) -> Self {
        Self { position, color }
    }
}

/// A vertex exactly as the GPU reads it: 2 floats of position, then 4 floats of linear color.
///
/// `#[repr(C)]` fixes the field order and leaves no padding, so the bytes match the vertex
/// buffer layout in `shape_pipeline.rs`. `Pod` ("plain old data") lets bytemuck view a
/// `&[GpuVertex]` as `&[u8]` without copying.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub(crate) struct GpuVertex {
    pub(crate) position: [f32; 2],
    pub(crate) color: [f32; 4],
}

impl From<Vertex> for GpuVertex {
    fn from(vertex: Vertex) -> Self {
        Self {
            position: vertex.position,
            color: vertex.color.to_linear(),
        }
    }
}

/// All shapes submitted during one frame.
#[derive(Debug, Default)]
pub(crate) struct ShapeBatch {
    vertices: Vec<GpuVertex>,
    indices: Vec<u32>,
}

impl ShapeBatch {
    /// Forgets the previous frame's shapes but keeps the memory.
    pub(crate) fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    /// Adds a triangle with a color per corner.
    pub(crate) fn push_triangle(&mut self, corners: [Vertex; 3]) {
        let first = self.next_index();
        self.vertices.extend(corners.map(GpuVertex::from));
        self.indices.extend([first, first + 1, first + 2]);
    }

    /// Adds an axis-aligned rectangle between `min` (bottom left) and `max` (top right).
    pub(crate) fn push_quad(&mut self, min: [f32; 2], max: [f32; 2], color: Color) {
        let first = self.next_index();
        let [left, bottom] = min;
        let [right, top] = max;
        let corners = [[left, top], [right, top], [left, bottom], [right, bottom]];
        self.vertices
            .extend(corners.map(|position| GpuVertex::from(Vertex::new(position, color))));
        // Both triangles go counter-clockwise on screen, like every other shape: GPUs use the
        // winding order to tell front faces from back faces.
        self.indices
            .extend([first, first + 2, first + 1, first + 1, first + 2, first + 3]);
    }

    /// The vertices of the frame so far.
    pub(crate) fn vertices(&self) -> &[GpuVertex] {
        &self.vertices
    }

    /// The indices of the frame so far, three per triangle.
    pub(crate) fn indices(&self) -> &[u32] {
        &self.indices
    }

    /// Whether nothing was submitted this frame.
    pub(crate) fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    /// The index the next pushed vertex will get.
    fn next_index(&self) -> u32 {
        u32::try_from(self.vertices.len()).expect("more than u32::MAX vertices in one frame")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle() -> [Vertex; 3] {
        [
            Vertex::new([0.0, 0.5], Color::RED),
            Vertex::new([-0.5, -0.5], Color::GREEN),
            Vertex::new([0.5, -0.5], Color::BLUE),
        ]
    }

    #[test]
    fn a_triangle_adds_three_vertices_and_three_indices() {
        let mut batch = ShapeBatch::default();
        batch.push_triangle(triangle());

        assert_eq!(batch.vertices().len(), 3);
        assert_eq!(batch.indices(), &[0, 1, 2]);
        assert_eq!(batch.vertices()[1].position, [-0.5, -0.5]);
    }

    #[test]
    fn vertex_colors_are_converted_to_linear() {
        let mut batch = ShapeBatch::default();
        let grey = Color::rgb(0.5, 0.5, 0.5);
        batch.push_quad([0.0, 0.0], [1.0, 1.0], grey);

        assert_eq!(batch.vertices()[0].color, grey.to_linear());
    }

    #[test]
    fn a_quad_shares_corners_between_its_two_triangles() {
        let mut batch = ShapeBatch::default();
        batch.push_quad([-1.0, -1.0], [1.0, 1.0], Color::WHITE);

        let positions: Vec<[f32; 2]> = batch.vertices().iter().map(|v| v.position).collect();
        assert_eq!(
            positions,
            vec![[-1.0, 1.0], [1.0, 1.0], [-1.0, -1.0], [1.0, -1.0]]
        );
        assert_eq!(batch.indices(), &[0, 2, 1, 1, 2, 3]);
    }

    #[test]
    fn indices_of_later_shapes_point_past_earlier_vertices() {
        let mut batch = ShapeBatch::default();
        batch.push_triangle(triangle());
        batch.push_quad([0.0, 0.0], [1.0, 1.0], Color::WHITE);

        assert_eq!(batch.vertices().len(), 7);
        assert_eq!(&batch.indices()[3..], &[3, 5, 4, 4, 5, 6]);
    }

    #[test]
    fn clear_empties_the_batch() {
        let mut batch = ShapeBatch::default();
        batch.push_triangle(triangle());
        assert!(!batch.is_empty());

        batch.clear();
        assert!(batch.is_empty());
        assert!(batch.vertices().is_empty());
    }

    #[test]
    fn a_gpu_vertex_is_six_tightly_packed_floats() {
        assert_eq!(std::mem::size_of::<GpuVertex>(), 6 * 4);
    }
}
