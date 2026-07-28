//! `pipeline2d` — the textured-quad pipeline for Raven's 640x480 virtual 2D
//! screen (R4a backend #1, v0).
//!
//! The oracle's 2D path is fixed-function: `RB_SetGL2D` sets a full-window
//! viewport, loads `qglOrtho(0, 640, 480, 0, 0, 1)` as the projection, and
//! every 2D draw thereafter emits quads in that 640x480 *virtual* space —
//! the viewport, not the coordinates, does the scale to the real resolution.
//! This module reproduces that as one WGSL pipeline: an ortho uniform in bind
//! group 0, a texture/sampler pair in bind group 1, and a vertex format of
//! `position` + `uv` + `color`.
//!
//! Source: `oracle/codemp/renderer/tr_backend.cpp:1266-1292` (`RB_SetGL2D`);
//! `oracle/codemp/game/q_shared.h:1029-1030` (`SCREEN_WIDTH`/`SCREEN_HEIGHT`)
//!
//! v0 scope: one built-in 1x1 white texture and one bind group. Real image
//! upload — resolving a [`ShaderHandle`] to an uploaded `ImageAsset` — lands
//! next wave, once `mp_renderer` retains decoded pixels. Until then a
//! `DrawStretchPic` renders as its flat vertex colour, which is enough to
//! prove the event -> execution -> pixels path.
//!
//! Colour space: the surface is typically an sRGB format, so wgpu encodes the
//! shader's linear output on write while the oracle wrote colour bytes
//! straight through. Matching Raven's exact ramp is a later fidelity item, not
//! a v0 blocker.

use std::collections::HashMap;
use std::mem::size_of;

use bytemuck::{Pod, Zeroable};
use wgpu::{BlendState, RenderPipeline, TextureView};

use crate::gpu::Gpu;

/// Raven `SCREEN_WIDTH` — the virtual 2D screen width every UI/HUD draw is
/// authored against.
///
/// Source: `oracle/codemp/game/q_shared.h:1029`
pub const SCREEN_WIDTH: f32 = 640.0;

/// Raven `SCREEN_HEIGHT`.
///
/// Source: `oracle/codemp/game/q_shared.h:1030`
pub const SCREEN_HEIGHT: f32 = 480.0;

/// Vertices per quad — two triangles, no index buffer (a 2D frame's quad count
/// is small enough that the index-buffer saving is not worth the second
/// buffer at v0).
const VERTICES_PER_QUAD: usize = 6;

/// Starting vertex-buffer capacity, in vertices; the buffer grows by doubling
/// when a frame needs more.
const INITIAL_VERTEX_CAPACITY: usize = 4 * 1024;

/// A screen-space rectangle in the 640x480 virtual space, `(x, y)` at the
/// top-left corner and `y` growing downward (Raven's 2D convention).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// A texture-coordinate rectangle given as opposite corners, matching
/// `DrawStretchPic`'s `s1`/`t1`/`s2`/`t2`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UvRect {
    pub s1: f32,
    pub t1: f32,
    pub s2: f32,
    pub t2: f32,
}

/// One vertex of the 2D pipeline. `#[repr(C)]` here is a GPU-layout
/// requirement (it must match the `VertexBufferLayout` below), not an ABI
/// seam.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Vertex2d {
    position: [f32; 2],
    uv: [f32; 2],
    color: [f32; 4],
}

const VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 3] =
    wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x4];

/// A contiguous span of the batch's vertices sharing one blend state — the
/// unit of one `draw` call. Consecutive quads with the same blend state merge
/// into one run, so a typical UI frame is a single draw.
#[derive(Clone, Copy, Debug)]
struct DrawRun {
    blend: BlendState,
    first_vertex: u32,
    vertex_count: u32,
}

/// The frame's accumulated 2D geometry. Owned by the executor and reused
/// across frames — [`QuadBatch::clear`] keeps the allocation.
#[derive(Default)]
pub struct QuadBatch {
    vertices: Vec<Vertex2d>,
    runs: Vec<DrawRun>,
}

impl QuadBatch {
    pub fn new() -> QuadBatch {
        QuadBatch::default()
    }

    /// Drops the frame's geometry, keeping the allocations for the next frame.
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.runs.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.runs.is_empty()
    }

    /// Number of `draw` calls [`Pipeline2d::draw`] will issue for this batch.
    pub fn run_count(&self) -> u32 {
        self.runs.len() as u32
    }

    /// Appends one screen-space quad, extending the tail run when `blend`
    /// matches it and opening a new run otherwise.
    pub fn push_quad(&mut self, rect: Rect, uv: UvRect, color: [f32; 4], blend: BlendState) {
        let (x0, y0) = (rect.x, rect.y);
        let (x1, y1) = (rect.x + rect.w, rect.y + rect.h);

        let top_left = Vertex2d {
            position: [x0, y0],
            uv: [uv.s1, uv.t1],
            color,
        };
        let top_right = Vertex2d {
            position: [x1, y0],
            uv: [uv.s2, uv.t1],
            color,
        };
        let bottom_right = Vertex2d {
            position: [x1, y1],
            uv: [uv.s2, uv.t2],
            color,
        };
        let bottom_left = Vertex2d {
            position: [x0, y1],
            uv: [uv.s1, uv.t2],
            color,
        };

        let first_vertex = self.vertices.len() as u32;
        self.vertices.extend_from_slice(&[
            top_left,
            top_right,
            bottom_right,
            top_left,
            bottom_right,
            bottom_left,
        ]);

        match self.runs.last_mut() {
            Some(run) if run.blend == blend => run.vertex_count += VERTICES_PER_QUAD as u32,
            _ => self.runs.push(DrawRun {
                blend,
                first_vertex,
                vertex_count: VERTICES_PER_QUAD as u32,
            }),
        }
    }
}

/// The 2D pipeline's GPU-side resources: shader module, layouts, the ortho
/// uniform, the placeholder white texture, the growable vertex buffer, and the
/// per-blend-state pipeline cache.
///
/// DEC-37 ruling 2/3: this is render-thread-owned state. Nothing here is
/// reachable from a trap query.
pub struct Pipeline2d {
    shader: wgpu::ShaderModule,
    pipeline_layout: wgpu::PipelineLayout,
    /// One pipeline per distinct blend state, built on first use. Backend #1
    /// resolves a stage's `GLS_*` bits with [`crate::blend`] and looks the
    /// result up here, so a shader script's blend mode costs one pipeline, not
    /// one per draw.
    pipelines: HashMap<BlendState, RenderPipeline>,
    surface_format: wgpu::TextureFormat,
    transform_bind_group: wgpu::BindGroup,
    texture_bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    vertex_capacity: usize,
}

impl Pipeline2d {
    /// Builds the pipeline's fixed resources against `gpu`'s device.
    pub fn new(gpu: &Gpu) -> Pipeline2d {
        let device = gpu.device();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("mp_renderer_gpu 2d shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/pipeline2d.wgsl").into()),
        });

        let transform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("mp_renderer_gpu 2d transform layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let texture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("mp_renderer_gpu 2d texture layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("mp_renderer_gpu 2d pipeline layout"),
            bind_group_layouts: &[Some(&transform_layout), Some(&texture_layout)],
            immediate_size: 0,
        });

        let transform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("mp_renderer_gpu 2d ortho uniform"),
            size: size_of::<[f32; 16]>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        gpu.queue().write_buffer(
            &transform_buffer,
            0,
            bytemuck::cast_slice(&virtual_screen_ortho()),
        );

        let transform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("mp_renderer_gpu 2d transform bind group"),
            layout: &transform_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: transform_buffer.as_entire_binding(),
            }],
        });

        let texture_bind_group =
            create_white_texture_bind_group(device, gpu.queue(), &texture_layout);

        let vertex_buffer = create_vertex_buffer(device, INITIAL_VERTEX_CAPACITY);

        Pipeline2d {
            shader,
            pipeline_layout,
            pipelines: HashMap::new(),
            surface_format: gpu.surface_format(),
            transform_bind_group,
            texture_bind_group,
            vertex_buffer,
            vertex_capacity: INITIAL_VERTEX_CAPACITY,
        }
    }

    /// Uploads `batch` and records one render pass over `target`, one draw per
    /// blend run. The pass loads (never clears) so it composites on top of
    /// whatever the frame's earlier passes left — `Gpu::begin_frame`'s clear
    /// today, the 3D scene pass once R4b lands. Returns the draw-call count.
    pub fn draw(&mut self, gpu: &Gpu, target: &TextureView, batch: &QuadBatch) -> u32 {
        if batch.is_empty() {
            return 0;
        }

        self.reserve(gpu, batch.vertices.len());
        gpu.queue().write_buffer(
            &self.vertex_buffer,
            0,
            bytemuck::cast_slice(&batch.vertices),
        );

        // Every blend state the batch uses must have a pipeline before the
        // pass borrows `self` immutably.
        for run in &batch.runs {
            self.ensure_pipeline(gpu, run.blend);
        }

        let mut encoder = gpu
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("mp_renderer_gpu 2d encoder"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("mp_renderer_gpu 2d pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            pass.set_bind_group(0, &self.transform_bind_group, &[]);
            pass.set_bind_group(1, &self.texture_bind_group, &[]);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

            for run in &batch.runs {
                let pipeline = self
                    .pipelines
                    .get(&run.blend)
                    .expect("2d pipeline was created for every run's blend state above");
                pass.set_pipeline(pipeline);
                pass.draw(run.first_vertex..run.first_vertex + run.vertex_count, 0..1);
            }
        }
        gpu.queue().submit(std::iter::once(encoder.finish()));

        batch.run_count()
    }

    /// Grows the vertex buffer (by doubling) when `needed` exceeds capacity.
    fn reserve(&mut self, gpu: &Gpu, needed: usize) {
        if needed <= self.vertex_capacity {
            return;
        }
        let mut capacity = self.vertex_capacity.max(1);
        while capacity < needed {
            capacity *= 2;
        }
        self.vertex_buffer = create_vertex_buffer(gpu.device(), capacity);
        self.vertex_capacity = capacity;
    }

    /// Builds and caches the pipeline for `blend` if it is not already there.
    fn ensure_pipeline(&mut self, gpu: &Gpu, blend: BlendState) {
        if self.pipelines.contains_key(&blend) {
            return;
        }
        let pipeline = gpu
            .device()
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("mp_renderer_gpu 2d pipeline"),
                layout: Some(&self.pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &self.shader,
                    entry_point: Some("vs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[Some(wgpu::VertexBufferLayout {
                        array_stride: size_of::<Vertex2d>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &VERTEX_ATTRIBUTES,
                    })],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    // `RB_SetGL2D` disables face culling for the whole 2D pass.
                    // Source: `oracle/codemp/renderer/tr_backend.cpp:1286`
                    cull_mode: None,
                    ..Default::default()
                },
                // No depth attachment: `RB_SetGL2D` sets `GLS_DEPTHTEST_DISABLE`,
                // so 2D draws composite purely in submission order.
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &self.shader,
                    entry_point: Some("fs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: self.surface_format,
                        blend: Some(blend),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: None,
            });
        self.pipelines.insert(blend, pipeline);
    }
}

/// The column-major orthographic matrix for `qglOrtho(0, 640, 480, 0, 0, 1)`,
/// adjusted for wgpu's `0..1` clip-space depth range (GL's was `-1..1`; every
/// 2D vertex sits at `z = 0`, which both ranges accept).
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:1277`
fn virtual_screen_ortho() -> [f32; 16] {
    [
        2.0 / SCREEN_WIDTH,
        0.0,
        0.0,
        0.0,
        //
        0.0,
        -2.0 / SCREEN_HEIGHT,
        0.0,
        0.0,
        //
        0.0,
        0.0,
        -1.0,
        0.0,
        //
        -1.0,
        1.0,
        0.0,
        1.0,
    ]
}

fn create_vertex_buffer(device: &wgpu::Device, capacity: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("mp_renderer_gpu 2d vertex buffer"),
        size: (capacity * size_of::<Vertex2d>()) as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

/// The v0 stand-in for a registered image: one opaque white texel, so
/// `texture * vertex_color` reduces to the vertex colour. Sampling is
/// linear/clamp, matching `R_RegisterShaderNoMip`'s `GL_LINEAR` + `GL_CLAMP`
/// 2D images.
fn create_white_texture_bind_group(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
) -> wgpu::BindGroup {
    let size = wgpu::Extent3d {
        width: 1,
        height: 1,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("mp_renderer_gpu 2d white texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &[0xff, 0xff, 0xff, 0xff],
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4),
            rows_per_image: Some(1),
        },
        size,
    );

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("mp_renderer_gpu 2d sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });

    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("mp_renderer_gpu 2d texture bind group"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blend::{blend_state_from_gls, ALPHA_BLEND, GLS_2D_DEFAULT};

    fn unit_uv() -> UvRect {
        UvRect {
            s1: 0.0,
            t1: 0.0,
            s2: 1.0,
            t2: 1.0,
        }
    }

    /// Applies the ortho matrix to a virtual-screen point, as the vertex
    /// shader does.
    fn to_clip(x: f32, y: f32) -> (f32, f32) {
        let m = virtual_screen_ortho();
        (m[0] * x + m[12], m[5] * y + m[13])
    }

    #[test]
    fn ortho_maps_the_virtual_screen_corners_to_clip_space() {
        // Top-left of the virtual screen is clip (-1, +1); y grows downward.
        assert_eq!(to_clip(0.0, 0.0), (-1.0, 1.0));
        assert_eq!(to_clip(SCREEN_WIDTH, SCREEN_HEIGHT), (1.0, -1.0));
        assert_eq!(to_clip(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0), (0.0, 0.0));
    }

    #[test]
    fn same_blend_quads_merge_into_one_run() {
        let mut batch = QuadBatch::new();
        let rect = Rect {
            x: 0.0,
            y: 0.0,
            w: 10.0,
            h: 10.0,
        };
        batch.push_quad(rect, unit_uv(), [1.0; 4], ALPHA_BLEND);
        batch.push_quad(
            rect,
            unit_uv(),
            [1.0; 4],
            blend_state_from_gls(GLS_2D_DEFAULT),
        );
        assert_eq!(batch.run_count(), 1);
        assert_eq!(batch.vertices.len(), 2 * VERTICES_PER_QUAD);
    }

    #[test]
    fn differing_blend_states_open_new_runs() {
        let mut batch = QuadBatch::new();
        let rect = Rect {
            x: 0.0,
            y: 0.0,
            w: 10.0,
            h: 10.0,
        };
        batch.push_quad(rect, unit_uv(), [1.0; 4], ALPHA_BLEND);
        batch.push_quad(rect, unit_uv(), [1.0; 4], blend_state_from_gls(0));
        assert_eq!(batch.run_count(), 2);
        assert_eq!(batch.runs[1].first_vertex, VERTICES_PER_QUAD as u32);

        batch.clear();
        assert!(batch.is_empty());
    }
}
