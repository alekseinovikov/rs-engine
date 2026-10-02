//! The render pipeline that draws [`ShapeBatch`](super::shapes::ShapeBatch)es.
//!
//! A *render pipeline* is the GPU's complete recipe for drawing: which shaders run, how the
//! vertex buffer's bytes split into shader inputs, how vertices form triangles, and how the
//! output color is blended into the image. It is compiled once, up front, because validating and
//! translating all of this is expensive; drawing then only says "use this pipeline".

use std::mem::size_of;

use super::buffer::GrowableBuffer;
use super::shapes::{GpuVertex, ShapeBatch};

/// The compiled pipeline and the buffers its vertices and indices are uploaded to.
#[derive(Debug)]
pub(crate) struct ShapePipeline {
    pipeline: wgpu::RenderPipeline,
    vertices: GrowableBuffer,
    indices: GrowableBuffer,
}

/// How a vertex buffer's bytes map to the shader's `VertexInput`: one [`GpuVertex`] per vertex,
/// position at offset 0 (`@location(0)`), color right after it (`@location(1)`).
const VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 2] = [
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x2,
        offset: 0,
        shader_location: 0,
    },
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x4,
        offset: size_of::<[f32; 2]>() as wgpu::BufferAddress,
        shader_location: 1,
    },
];

impl ShapePipeline {
    /// Compiles the shader and the pipeline for images of `format`.
    pub(crate) fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shapes shader"),
            // `include_str!` embeds the file in the binary at compile time, so the shader
            // cannot go missing at runtime.
            source: wgpu::ShaderSource::Wgsl(include_str!("shapes.wgsl").into()),
        });
        // The layout lists the bind groups (uniforms, textures) the shaders use. We use none
        // yet; the camera uniform arrives in M3.
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("shapes pipeline layout"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("shapes pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: size_of::<GpuVertex>() as wgpu::BufferAddress,
                    // A new vertex for every vertex (not per instance; instancing is M3).
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &VERTEX_ATTRIBUTES,
                })],
            },
            primitive: wgpu::PrimitiveState {
                // Every three indices form one triangle.
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                // 2D shapes are never seen from behind, so nothing is culled; a shape submitted
                // clockwise still draws.
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    // Classic transparency: result = src × src.alpha + dst × (1 − src.alpha).
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        Self {
            pipeline,
            vertices: GrowableBuffer::new(
                device,
                "shape vertex buffer",
                wgpu::BufferUsages::VERTEX,
            ),
            indices: GrowableBuffer::new(device, "shape index buffer", wgpu::BufferUsages::INDEX),
        }
    }

    /// Uploads the batch. Call before the render pass begins.
    pub(crate) fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        batch: &ShapeBatch,
    ) {
        if batch.is_empty() {
            return;
        }
        self.vertices
            .write(device, queue, bytemuck::cast_slice(batch.vertices()));
        self.indices
            .write(device, queue, bytemuck::cast_slice(batch.indices()));
    }

    /// Records the draw call for the batch uploaded by [`ShapePipeline::upload`].
    pub(crate) fn draw(&self, pass: &mut wgpu::RenderPass<'_>, batch: &ShapeBatch) {
        if batch.is_empty() {
            return;
        }
        let index_count = u32::try_from(batch.indices().len()).expect("index count fits in u32");
        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, self.vertices.buffer().slice(..));
        pass.set_index_buffer(self.indices.buffer().slice(..), wgpu::IndexFormat::Uint32);
        // All shapes of the frame in one draw call: indices 0..index_count, base vertex 0,
        // one instance.
        pass.draw_indexed(0..index_count, 0, 0..1);
    }
}
