// The shader for colored shapes. WGSL is WebGPU's shading language; wgpu translates it to the
// platform's own (Metal Shading Language on macOS).
//
// The GPU runs `vs_main` once per vertex and `fs_main` once per pixel the triangles cover.
// Between the two, it *interpolates* every output of the vertex shader across the triangle:
// a pixel halfway between a red and a blue corner receives a color halfway between them.

// One vertex as stored in the vertex buffer. The `@location` numbers match the
// `shader_location`s of the vertex buffer layout in `shape_pipeline.rs`.
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
}

// What the vertex shader hands to the rasterizer.
struct VertexOutput {
    // `@builtin(position)` is the one output the GPU itself needs: where the vertex lands, in
    // clip space (x and y from -1 to 1). The fourth component, `w`, is 1 for 2D.
    @builtin(position) clip_position: vec4<f32>,
    // Everything else is interpolated across the triangle and passed to the fragment shader.
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    // Positions already are in clip space in M2; from M3 on a camera matrix transforms them.
    out.clip_position = vec4<f32>(in.position, 0.0, 1.0);
    out.color = in.color;
    return out;
}

// The returned color is linear; the sRGB surface encodes it when it writes the pixel.
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
