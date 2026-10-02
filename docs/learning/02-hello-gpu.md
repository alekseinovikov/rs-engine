# 02 — Hello GPU

**Milestone:** M2 · **Tag:** `m2-hello-gpu`

In this milestone the engine got its first pixels from the GPU. Every frame it clears the window
and draws whatever colored triangles and rectangles the game submitted through `ctx.gfx`. All of
it goes through wgpu, a Rust implementation of the WebGPU standard that runs on Metal on macOS.

## What we built

```
crates/engine/src/gfx/
├── mod.rs              Gfx: the game's drawing API; begin_frame / end_frame; the render pass
├── color.rs            Color: sRGB colors and their conversion to linear
├── shapes.rs           Vertex, GpuVertex, ShapeBatch: shapes collected on the CPU
├── buffer.rs           GrowableBuffer: a GPU buffer that doubles to fit the frame
├── gpu.rs              Gpu: adapter, device, queue
├── surface.rs          WindowSurface: the swapchain, resize, vsync, lost surfaces
├── shape_pipeline.rs   ShapePipeline: the render pipeline and its buffers
└── shapes.wgsl         the vertex and fragment shaders
crates/engine/examples/triangle.rs
```

Outside `gfx`, `Config` gained `vsync`, `Context` gained the public field `gfx`, and `app.rs` now
creates the renderer in `resumed` and wraps `game.draw` in `begin_frame` / `end_frame`.

`cargo run -p engine --example triangle` shows a triangle with red, green and blue corners, a
yellow rectangle and a half-transparent white rectangle on top of both. C cycles the background
color, V toggles vsync, F toggles a 30 FPS limit.

## Key concepts

### The GPU is a second computer

The CPU and the GPU are separate processors with separate memory. The CPU does not "call" the
GPU. It **records** a list of commands, **submits** the list to a queue, and goes on with its own
work while the GPU executes the commands later. Almost every design decision in a renderer follows
from this:

- data the GPU needs (vertices, textures) must be **uploaded** into GPU buffers first;
- expensive setup (compiling shaders, validating state) is done **once**, in advance, in a
  pipeline object;
- the fewer separate commands per frame, the better. This is why we draw the whole frame with
  **one draw call**.

### Instance, adapter, device, queue

wgpu follows WebGPU's four objects (see `gpu.rs`):

- **Instance**: the entry point. It loads the platform API (Metal, Vulkan, DX12, GL).
- **Adapter**: one physical GPU and what it can do. We ask for one that is compatible with our
  window's surface and prefer the high-performance one.
- **Device**: our private connection to that adapter. Every buffer, shader and pipeline is created
  through it.
- **Queue**: where command buffers are submitted and small uploads (`write_buffer`) are
  scheduled.

Requesting an adapter and a device is `async`, because in a browser it really is. On desktop it
finishes immediately, so `pollster::block_on` waits for it without needing an async runtime.

### The surface and the swapchain

A **surface** is the GPU's view of the window. It owns a small ring of images, the
**swapchain**. Every frame we take the next free image, draw into it and **present** it. The
compositor shows that image on screen while we draw the next frame into another one.

The surface must be **configured** with a size, a pixel format and a present mode, and configured
again whenever the window is resized. In wgpu 30, `get_current_texture` returns an enum that
tells us what happened, and `surface.rs` handles every case:

| Status | Meaning | What we do |
|---|---|---|
| `Success` | here is an image | draw |
| `Suboptimal` | the image works but no longer matches the window exactly | draw, reconfigure before the next frame |
| `Timeout`, `Occluded` | no image freed up in time, or the window is hidden | skip the frame |
| `Outdated` | the window changed under the configuration | reconfigure, skip |
| `Lost` | the surface is gone (display change, driver reset) | recreate it from the window, skip |
| `Validation` | an API misuse was caught | log an error, skip |

### Present modes and vsync

With **vsync** (`AutoVsync`: `FifoRelaxed` if available, otherwise `Fifo`), `present` waits for
the display's next refresh. The frame rate matches the display (60 or 120 Hz), the CPU sleeps
instead of spinning, and the image never **tears** (shows the top of one frame and the bottom of
the next). Without vsync (`AutoNoVsync`: `Immediate`, then `Mailbox`, then `Fifo`), frames are
shown as soon as they are ready. Our `max_fps` limiter from M1 is still there: vsync waits on the GPU side, the limiter
sleeps on the CPU side, and the two combine. Try V and F in the example.

### How a frame works

`Gfx::end_frame` is one frame in miniature:

```
texture  = surface.acquire()                    the next swapchain image (or skip the frame)
upload     vertices and indices                 queue.write_buffer, staged for the next submit
encoder  = device.create_command_encoder()
pass     = encoder.begin_render_pass(clear)     clear the image to the clear color
pass.set_pipeline / set_vertex_buffer / set_index_buffer / draw_indexed
drop(pass)                                      the pass borrows the encoder; end it first
queue.submit(encoder.finish())                  send the recorded commands to the GPU
queue.present(texture)                          hand the image to the compositor
```

### Vertices, indices and one draw call

A GPU draws **triangles** from a list of **vertices**. An **index buffer** says which three
vertices form each triangle. A rectangle becomes 4 vertices and 6 indices, with two corners
shared, instead of 6 vertices:

```
 0 ──── 1      triangle A: 0, 2, 1
 │ A  ╱ │      triangle B: 1, 2, 3
 │  ╱ B │
 2 ──── 3
```

`ShapeBatch` appends every shape of the frame to one vertex `Vec` and one index `Vec`. Each new
shape's indices are offset by the number of vertices already in the batch (the test
`indices_of_later_shapes_point_past_earlier_vertices` checks this). Triangles are indexed too
(0, 1, 2), so all shapes share one pipeline and one `draw_indexed` call. This is batching in its
simplest form. M3 takes it further with instancing.

`GpuVertex` is `#[repr(C)]` and derives bytemuck's `Pod`, so `bytemuck::cast_slice` can view
`&[GpuVertex]` as the raw `&[u8]` the GPU wants, without copying and without `unsafe` in our
code. The bytes must match the **vertex buffer layout** in `shape_pipeline.rs`: a stride of 24
bytes, the position (`Float32x2`) at offset 0 and the color (`Float32x4`) at offset 8.

### Shaders: vertex, rasterizer, fragment

`shapes.wgsl` holds two small programs that run on the GPU:

1. The **vertex shader** (`vs_main`) runs once per vertex and returns its position in **clip
   space**, plus any values to pass on (here the color).
2. The **rasterizer**, fixed hardware rather than our code, works out which pixels each triangle
   covers and **interpolates** the vertex outputs across the triangle. A pixel near the red
   corner gets mostly red.
3. The **fragment shader** (`fs_main`) runs once per covered pixel and returns its color.

The `@location(n)` numbers connect the pieces: vertex buffer attribute n → vertex shader input
n, and vertex shader output n → fragment shader input n.

### The render pipeline

A **render pipeline** bundles everything about *how* to draw: the shaders, the vertex layout,
the primitive topology (a triangle list), culling, the output format and the blend mode. Creating
one validates and compiles all of it, which is slow, so we create it once in
`ShapePipeline::new`. Drawing then only says "use this pipeline". Changing any of that state
means a different pipeline. This is why engines keep a small number of them.

### Clip space

The vertex shader's output is in **clip space**: `x` from −1 (left edge) to 1 (right edge), `y`
from −1 (bottom) to 1 (top), whatever the window size. That is why the shapes stretch when you
resize the window. M2's `draw_triangle` and `draw_quad` take clip-space coordinates directly,
which is a temporary API. In M3 a camera matrix maps world pixels (+y down) into clip space.

### sRGB vs linear

Displays do not map numbers to brightness linearly. sRGB, the standard every image editor and CSS
uses, spends more of its steps on dark shades, so sRGB 0.5 is only about 21 % of the light of 1.0.
Mixing light (interpolating, blending), however, is only correct on **linear** values. Our
pipeline keeps the two straight:

1. the game writes colors in sRGB (`Color::from_hex(0x1E1E2E)`), exactly as in an image editor;
2. `Color::to_linear` converts them on the CPU, when a vertex enters the batch;
3. the GPU interpolates and blends linear values;
4. the surface's format is `Bgra8UnormSrgb`, so the GPU converts each pixel back to sRGB as it
   writes it.

The conversion is the sRGB transfer function in `color.rs`: a straight line near black, a power
curve with exponent 2.4 above 0.04045.

### Alpha blending

The pipeline's blend state is `ALPHA_BLENDING`: `result = src × src.a + dst × (1 − src.a)`.
Shapes are drawn in submission order, so the white rectangle submitted last appears over the other
two. Ordering by layer arrives in M3.

## Where to look in the code

| Concept | Where |
|---|---|
| Instance, adapter, device, queue | `crates/engine/src/gfx/gpu.rs`, `Gfx::new` in `gfx/mod.rs` |
| Swapchain configuration, vsync, statuses | `crates/engine/src/gfx/surface.rs` |
| The frame: acquire, upload, render pass, submit, present | `Gfx::end_frame` in `gfx/mod.rs` |
| Vertices, indices, the batch | `crates/engine/src/gfx/shapes.rs` |
| Vertex layout, pipeline state, blending | `crates/engine/src/gfx/shape_pipeline.rs` |
| Shaders and interpolation | `crates/engine/src/gfx/shapes.wgsl` |
| sRGB → linear | `crates/engine/src/gfx/color.rs` |
| Buffer growth | `crates/engine/src/gfx/buffer.rs` |
| Renderer creation, resize, begin/end frame | `App::start`, `App::frame`, `window_event` in `app.rs` |
| Backing off after skipped frames | `FramePacer::frame_skipped` in `crates/engine/src/pacing.rs` |

## Pitfalls

- **A zero-sized surface.** Minimizing a window resizes it to 0×0, and wgpu panics on
  configuring a zero-sized surface. `WindowSurface` keeps the size, skips the `configure` call and
  skips frames until the window is restored.
- **A hidden window spins the CPU.** On macOS a minimized or covered window makes
  `get_current_texture` return `Occluded` immediately. Our loop runs with `ControlFlow::Poll`, so
  without a pause it would retry at full speed and keep a core at 100 % while showing nothing.
  `end_frame` reports a skipped frame, and `FramePacer::frame_skipped` waits 100 ms before the
  next attempt (0.3 % CPU measured). The code review caught this one, not the first run.
- **Forgetting to reconfigure on resize.** The swapchain keeps its old size, and the image is
  stretched (macOS) or the surface reports `Outdated` (elsewhere).
- **Double gamma, or none.** Writing sRGB values to an sRGB surface without converting them means
  they are encoded twice, and everything looks washed out. Converting them but writing to a
  non-sRGB surface makes everything too dark. Exactly one conversion each way is correct.
- **Configuring while holding a frame.** `surface.configure` panics if a `SurfaceTexture` from
  that surface is still alive. This is why a `Suboptimal` frame is drawn first and the
  reconfiguration waits for the next `acquire`.
- **The render pass borrows the encoder.** `encoder.finish()` does not compile while the pass is
  alive, hence the extra `{ … }` block in `end_frame`.
- **Buffer sizes.** `write_buffer` needs sizes and offsets that are multiples of 4 bytes. A
  `GpuVertex` (24 bytes) and a `u32` index (4 bytes) always are. `u16` indices with an odd count
  would not be.
- **`event_loop.exit()` is a request, not an immediate stop.** In M1 the log printed "Esc pressed,
  quitting" *before* the game logged "pressed Escape". winit finishes the current iteration after
  `exit()`, so one more frame ran, and its tick saw the press. Anything that must happen exactly
  once before quitting (saving the game in M11, for example) has to account for that last frame.

## Experiments to try

1. In `shapes.wgsl`, return `vec4<f32>(in.color.rgb, 0.5)` from `fs_main`: every shape becomes
   half-transparent and the background shows through.
2. In `GpuVertex::from` (`shapes.rs`), replace `vertex.color.to_linear()` with
   `[vertex.color.r, vertex.color.g, vertex.color.b, vertex.color.a]`.
   The backgrounds turn lighter and the triangle's gradient changes. This is double gamma.
3. In `surface.rs`, make `choose_format` return the first format *without* sRGB (`!format.is_srgb()`)
   and compare the colors with experiment 2.
4. Swap two indices of the quad in `push_quad`, then set `cull_mode: Some(wgpu::Face::Back)` in
   `shape_pipeline.rs`: one triangle of the rectangle disappears. Why?
5. Change the topology to `PrimitiveTopology::LineList` and see how the same indices are read as
   pairs.
6. Set `MIN_CAPACITY` in `buffer.rs` to 16 and run with `RUST_LOG=engine=debug` to watch the
   buffers double.
7. Run with `WGPU_BACKEND=gl` to use OpenGL instead of Metal. The log shows the backend, and the
   picture should look the same.
8. Write an invalid WGSL line and run the example: the validation panic names the line and the
   problem.

## Further reading

- [Learn Wgpu](https://sotrh.github.io/learn-wgpu/): the same steps in tutorial form (check the
  wgpu version it targets; the API changes between major versions)
- [WebGPU Fundamentals](https://webgpufundamentals.org/): the concepts, with interactive diagrams
- The [WGSL specification](https://www.w3.org/TR/WGSL/) and the
  [WebGPU specification](https://www.w3.org/TR/webgpu/)
- John Novak, [What every coder should know about gamma](https://blog.johnnovak.net/2016/09/21/what-every-coder-should-know-about-gamma/)
- wgpu 30 docs: [`CurrentSurfaceTexture`](https://docs.rs/wgpu/30/wgpu/enum.CurrentSurfaceTexture.html)
  and [`PresentMode`](https://docs.rs/wgpu/30/wgpu/enum.PresentMode.html)
