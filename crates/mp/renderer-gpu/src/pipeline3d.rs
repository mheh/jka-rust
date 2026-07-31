//! `pipeline3d` — the lightmapped world pipeline for Raven's BSP surfaces
//! (R4 world backend, wave B).
//!
//! `mp_renderer`'s CPU frontend walks the PVS, culls, and hands this side a
//! sorted `DrawSurf` list over `WorldAsset::surfaces`. This module is the GPU
//! half: it packs every surface's vertices and indices into one buffer pair,
//! builds the view/projection clip matrix, and draws each surface with its
//! diffuse and lightmap textures under a depth test.
//!
//! One clip matrix (group 0), a diffuse-plus-lightmap texture pair (group 1),
//! and a per-surface `has_lightmap` flag (group 2, dynamic offset) drive the
//! `world.wgsl` shader. The fragment output is `diffuse.rgb * lightmap.rgb` on
//! a lightmapped surface, or `diffuse.rgb * vertex_color.rgb` where no lightmap
//! exists.
//!
//! Out of this wave: tcMod, animMap, rgbGen waves, fog, and multi-stage
//! shaders. Each is counted and warned once in the `frame_exec` `Warned` style.
//!
//! DEC-37 ruling 2/3: this is render-thread-owned state. Nothing here is
//! reachable from a trap query.

use std::collections::HashMap;
use std::mem::size_of;

use bytemuck::{Pod, Zeroable};
use mp_engine_qcommon::qfiles::draw_vert_t::drawVert_t;
use mp_renderer::render_state::image_asset::ImageHandle;
use mp_renderer::render_state::placeholders::WorldAsset;
use mp_renderer::render_state::render_assets::RenderAssets;
use mp_renderer::render_state::shader_asset::{ShaderAsset, ShaderHandle};
use mp_renderer::tr_bsp::{FaceVertex, SurfaceData};
use mp_renderer::tr_local::color_gen_t::colorGen_t;
use mp_renderer::tr_main::{DrawSurf, R_DecomposeSort, SurfaceGeometry, WorldSurfaceRef};
use mp_renderer::tr_shader::{GLS_DEPTHFUNC_EQUAL, GLS_DEPTHMASK_TRUE};
use wgpu::{BlendState, RenderPipeline, TextureView};

use crate::blend::blend_state_from_gls;
use crate::gpu::Gpu;
use crate::gpu_images::GpuImages;

/// The GPU vertex the three CPU surface shapes converge on. `FaceVertex`
/// (`tr_bsp`), `drawVert_t` (`qfiles`), and grid `drawVert_t` verts
/// (`tr_curve`) all collapse to this row. Lightmap style 0 and color style 0
/// are kept. The `drawVert_t` normal is dropped for this wave — the world
/// backend does no per-vertex lighting yet.
///
/// `#[repr(C)]` here is a GPU-layout requirement (it must match the
/// `VertexBufferLayout` below), not an ABI seam.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:810-812`
/// (`srfSurfaceFace_t` points row), `oracle/codemp/qcommon/qfiles.h:514-520`
/// (`drawVert_t`)
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct WorldVertex {
    position: [f32; 3],
    st: [f32; 2],
    lightmap_st: [f32; 2],
    color: [u8; 4],
}

impl WorldVertex {
    /// Converges a `FaceVertex` (`SF_FACE` points row, no normal) into the GPU
    /// vertex, keeping lightmap style 0 and color style 0.
    pub fn from_face_vertex(v: &FaceVertex) -> WorldVertex {
        WorldVertex {
            position: v.xyz,
            st: v.st,
            lightmap_st: v.lightmap[0],
            color: v.color[0],
        }
    }

    /// Converges a `drawVert_t` (grid and triangle-soup verts) into the GPU
    /// vertex, keeping lightmap style 0 and color style 0. The normal is
    /// dropped for this wave.
    pub fn from_draw_vert(v: &drawVert_t) -> WorldVertex {
        WorldVertex {
            position: v.xyz,
            st: v.st,
            lightmap_st: v.lightmap[0],
            color: v.color[0],
        }
    }
}

const VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
    0 => Float32x3,
    1 => Float32x2,
    2 => Float32x2,
    3 => Unorm8x4,
];

/// One surface's slice of the concatenated world buffers, addressable by the
/// `WorldSurfaceRef` index. `base_vertex` is added to every index by
/// `draw_indexed`, so a surface's indices stay 0-based within its own vertex
/// block. A `Skip` or `Flare` surface draws nothing and carries a zero
/// `index_count`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SurfaceRange {
    pub base_vertex: i32,
    pub first_index: u32,
    pub index_count: u32,
}

impl SurfaceRange {
    /// The empty range a `Skip` or `Flare` surface gets.
    const EMPTY: SurfaceRange = SurfaceRange {
        base_vertex: 0,
        first_index: 0,
        index_count: 0,
    };
}

/// The world's uploaded geometry: one concatenated vertex buffer, one index
/// buffer, and a per-surface range vector parallel to `WorldAsset::surfaces`.
pub struct WorldGeometry {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    ranges: Vec<SurfaceRange>,
}

impl WorldGeometry {
    /// Packs `world`'s surfaces into GPU buffers. An empty world (no surfaces,
    /// or every surface a `Skip`/`Flare`) still creates one-element buffers so
    /// the buffer handles are always valid.
    pub fn upload(gpu: &Gpu, world: &WorldAsset) -> WorldGeometry {
        let (vertices, indices, ranges) = build_world_mesh(world);

        // wgpu rejects a zero-size buffer, so an empty world falls back to one
        // zeroed element in each buffer.
        let vertex_fallback = [WorldVertex::zeroed()];
        let vertex_bytes = if vertices.is_empty() {
            bytemuck::cast_slice(&vertex_fallback)
        } else {
            bytemuck::cast_slice(&vertices)
        };
        let index_fallback = [0u32];
        let index_bytes = if indices.is_empty() {
            bytemuck::cast_slice(&index_fallback)
        } else {
            bytemuck::cast_slice(&indices)
        };

        let vertex_buffer = create_buffer(
            gpu.device(),
            "mp_renderer_gpu world vertex buffer",
            vertex_bytes,
            wgpu::BufferUsages::VERTEX,
        );
        let index_buffer = create_buffer(
            gpu.device(),
            "mp_renderer_gpu world index buffer",
            index_bytes,
            wgpu::BufferUsages::INDEX,
        );

        WorldGeometry {
            vertex_buffer,
            index_buffer,
            ranges,
        }
    }

    /// The range for surface `index`, or the empty range when `index` is out of
    /// bounds.
    pub fn range(&self, index: u32) -> SurfaceRange {
        self.ranges
            .get(index as usize)
            .copied()
            .unwrap_or(SurfaceRange::EMPTY)
    }

    /// The number of surface ranges — the same count as `WorldAsset::surfaces`.
    pub fn surface_count(&self) -> usize {
        self.ranges.len()
    }
}

/// Packs `world`'s surfaces into a vertex list, an index list, and one range
/// per surface. Faces carry points plus indices. Triangle soups carry verts
/// plus indexes. Grids carry a `width` by `height` vertex lattice and emit two
/// triangles per cell in row-major order. `Skip` and `Flare` draw nothing.
///
/// This is the pure half of [`WorldGeometry::upload`], split out for the unit
/// tests.
pub fn build_world_mesh(world: &WorldAsset) -> (Vec<WorldVertex>, Vec<u32>, Vec<SurfaceRange>) {
    let mut vertices: Vec<WorldVertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut ranges: Vec<SurfaceRange> = Vec::with_capacity(world.surfaces.len());

    for surface in &world.surfaces {
        let base_vertex = vertices.len() as i32;
        let first_index = indices.len() as u32;

        match &surface.data {
            SurfaceData::Face(face) => {
                for point in &face.points {
                    vertices.push(WorldVertex::from_face_vertex(point));
                }
                for &index in &face.indices {
                    indices.push(index as u32);
                }
            }

            SurfaceData::Triangles(tris) => {
                for vert in &tris.verts {
                    vertices.push(WorldVertex::from_draw_vert(vert));
                }
                for &index in &tris.indexes {
                    indices.push(index as u32);
                }
            }

            SurfaceData::Grid(grid) => {
                for vert in &grid.verts {
                    vertices.push(WorldVertex::from_draw_vert(vert));
                }
                indices.extend(grid_indices(grid.width, grid.height));
            }

            // A skip tag and a flare both draw no geometry.
            SurfaceData::Skip | SurfaceData::Flare(_) => {}
        }

        let index_count = indices.len() as u32 - first_index;
        ranges.push(SurfaceRange {
            base_vertex,
            first_index,
            index_count,
        });
    }

    (vertices, indices, ranges)
}

/// The row-major triangle indices for a `width` by `height` grid lattice, two
/// triangles per cell. The indices are 0-based within the grid's own vertex
/// block. A grid narrower or shorter than two verts has no cell, so no
/// triangle.
///
/// This v0 emits the full `width` by `height` lattice. `RB_SurfaceGrid`
/// subsamples the lattice through `widthTable`/`heightTable` from the surface
/// LOD error and splits a run at `SHADER_MAX_VERTEXES`. We drop LOD for now.
/// Source: `oracle/codemp/renderer/tr_surface.cpp:1572` (indices at `:1736-1755`)
//TODO: Port RB_SurfaceGrid LOD subsample (widthTable/heightTable + SHADER_MAX_VERTEXES split)
// Source: oracle/codemp/renderer/tr_surface.cpp:1597-1755
pub fn grid_indices(width: i32, height: i32) -> Vec<u32> {
    if width < 2 || height < 2 {
        return Vec::new();
    }
    let w = width as u32;
    let h = height as u32;
    let mut indices: Vec<u32> = Vec::with_capacity(((w - 1) * (h - 1) * 6) as usize);

    for y in 0..h - 1 {
        for x in 0..w - 1 {
            let top_left = y * w + x;
            let top_right = top_left + 1;
            let bottom_left = (y + 1) * w + x;
            let bottom_right = bottom_left + 1;

            indices.push(top_left);
            indices.push(bottom_left);
            indices.push(top_right);

            indices.push(top_right);
            indices.push(bottom_left);
            indices.push(bottom_right);
        }
    }
    indices
}

/// The GL-to-wgpu depth correction matrix in column-major storage. GL's
/// projection lands NDC z in -1..1. wgpu clip space wants 0..1, so this remaps
/// `z2 = 0.5 * z + 0.5 * w`.
///
/// Source: `oracle/codemp/renderer/tr_main.cpp:494-559` (`R_SetupProjection`
/// builds the -1..1 GL frustum this corrects)
pub fn depth_correction() -> [f32; 16] {
    [
        1.0, 0.0, 0.0, 0.0, // column 0
        0.0, 1.0, 0.0, 0.0, // column 1
        0.0, 0.0, 0.5, 0.0, // column 2
        0.0, 0.0, 0.5, 1.0, // column 3
    ]
}

/// The final clip matrix `correction * projection * model`, in column-major
/// storage. `model` is the view orientation `R_RotateForViewer` builds
/// (`viewParms_t.world.modelMatrix`). `projection` is
/// `viewParms_t.projectionMatrix`. The result maps a world-space position
/// straight to wgpu clip space.
pub fn world_clip_matrix(model: &[f32; 16], projection: &[f32; 16]) -> [f32; 16] {
    let mvp = mat4_mul(projection, model);
    mat4_mul(&depth_correction(), &mvp)
}

/// Column-major 4x4 matrix product `a * b`. Element `[col * 4 + row]` follows
/// GL's storage order, matching `viewParms_t.projectionMatrix`.
fn mat4_mul(a: &[f32; 16], b: &[f32; 16]) -> [f32; 16] {
    let mut out = [0.0f32; 16];
    for col in 0..4 {
        for row in 0..4 {
            let mut sum = 0.0f32;
            for k in 0..4 {
                sum += a[k * 4 + row] * b[col * 4 + k];
            }
            out[col * 4 + row] = sum;
        }
    }
    out
}

/// Column-major 4x4 matrix times a column vector `m * v`. The z-remap tests
/// use it to check where the near and far planes land.
#[cfg(test)]
fn mat4_mul_vec(m: &[f32; 16], v: [f32; 4]) -> [f32; 4] {
    let mut out = [0.0f32; 4];
    for row in 0..4 {
        let mut sum = 0.0f32;
        for col in 0..4 {
            sum += m[col * 4 + row] * v[col];
        }
        out[row] = sum;
    }
    out
}

/// A `Depth32Float` depth buffer sized to the render target, recreated on
/// resize.
pub struct DepthTexture {
    view: TextureView,
    width: u32,
    height: u32,
}

impl DepthTexture {
    /// Creates the depth texture at `width` by `height` (both clamped to at
    /// least 1, since wgpu rejects a zero extent).
    pub fn new(gpu: &Gpu, width: u32, height: u32) -> DepthTexture {
        let width = width.max(1);
        let height = height.max(1);
        let texture = gpu.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("mp_renderer_gpu world depth texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        DepthTexture {
            view,
            width,
            height,
        }
    }

    /// Recreates the depth texture if `width` by `height` differs from the
    /// current size.
    pub fn resize(&mut self, gpu: &Gpu, width: u32, height: u32) {
        let width = width.max(1);
        let height = height.max(1);
        if self.width == width && self.height == height {
            return;
        }
        *self = DepthTexture::new(gpu, width, height);
    }
}

/// The per-surface uniform stride for the dynamic-offset flags buffer. wgpu
/// requires a dynamic offset that is a multiple of
/// `min_uniform_buffer_offset_alignment`, which is at most 256 on every
/// backend, so 256 is always legal.
const SURFACE_FLAGS_STRIDE: u64 = 256;

/// The per-surface uniform the fragment shader reads through group 2. Only
/// `has_lightmap` matters. The padding rounds the write size to 16 bytes.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct SurfaceFlagsGpu {
    has_lightmap: u32,
    _pad: [u32; 3],
}

/// The out-of-scope shader features the world backend cannot render yet,
/// tracked so each one logs once per process rather than once per surface.
#[derive(Clone, Copy, Debug)]
enum Warned {
    TcMod,
    AnimMap,
    RgbGenWave,
    Fog,
    MultiStage,
}

impl Warned {
    const COUNT: usize = 5;

    fn slot(self) -> usize {
        match self {
            Warned::TcMod => 0,
            Warned::AnimMap => 1,
            Warned::RgbGenWave => 2,
            Warned::Fog => 3,
            Warned::MultiStage => 4,
        }
    }

    fn describe(self) -> &'static str {
        match self {
            Warned::TcMod => "skips tcMod on a world stage — not applied yet",
            Warned::AnimMap => "skips animMap on a world stage — first frame only",
            Warned::RgbGenWave => "skips rgbGen wave on a world stage — not applied yet",
            Warned::Fog => "skips fog on a world shader — not applied yet",
            Warned::MultiStage => "draws a multi-stage world shader as diffuse plus lightmap only",
        }
    }
}

/// What one [`Pipeline3d::draw`] call did.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WorldStats {
    /// World surfaces drawn (one indexed draw each).
    pub surfaces_drawn: u32,
    /// `draw_indexed` calls issued — the same as `surfaces_drawn` this wave.
    pub draw_calls: u32,
    /// World surfaces with a real lightmap bound.
    pub lightmapped: u32,
    /// Non-world draw-surf entries skipped (entity models, polys, and so on).
    pub skipped_non_world: u32,
    /// World surfaces whose range was empty (`Skip`/`Flare`), so nothing drew.
    pub empty_surfaces: u32,
}

/// The pipeline cache key: one pipeline per distinct blend state and depth
/// state. `BlendState` is `Hash`; the two depth choices are booleans.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct PipelineKey {
    blend: BlendState,
    depth_equal: bool,
    depth_write: bool,
}

/// The resolved state one world surface draws with, collected before the render
/// pass so the pass borrows `self` immutably.
struct WorldDrawItem {
    range: SurfaceRange,
    key: PipelineKey,
    diffuse: Option<ImageHandle>,
    lightmap: Option<ImageHandle>,
    has_lightmap: bool,
}

/// The world pipeline's GPU-side resources: shader module, layouts, the clip
/// matrix uniform, the per-surface flags buffer, the depth texture, and the
/// per-state pipeline cache. Textures are not owned here — [`GpuImages`] owns
/// them, and `draw` borrows it to build each surface's bind group.
///
/// DEC-37 ruling 2/3: this is render-thread-owned state. Nothing here is
/// reachable from a trap query.
pub struct Pipeline3d {
    shader: wgpu::ShaderModule,
    pipeline_layout: wgpu::PipelineLayout,
    texture_layout: wgpu::BindGroupLayout,
    pipelines: HashMap<PipelineKey, RenderPipeline>,
    surface_format: wgpu::TextureFormat,
    globals_buffer: wgpu::Buffer,
    globals_bind_group: wgpu::BindGroup,
    flags_layout: wgpu::BindGroupLayout,
    flags_buffer: wgpu::Buffer,
    flags_bind_group: wgpu::BindGroup,
    flags_capacity: usize,
    depth: DepthTexture,
    warned: [bool; Warned::COUNT],
}

impl Pipeline3d {
    /// Builds the pipeline's fixed resources against `gpu`'s device.
    pub fn new(gpu: &Gpu) -> Pipeline3d {
        let device = gpu.device();
        let (width, height) = gpu.surface_size();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("mp_renderer_gpu world shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/world.wgsl").into()),
        });

        let globals_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("mp_renderer_gpu world globals layout"),
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
            label: Some("mp_renderer_gpu world texture layout"),
            entries: &[
                texture_entry(0),
                sampler_entry(1),
                texture_entry(2),
                sampler_entry(3),
            ],
        });

        let flags_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("mp_renderer_gpu world flags layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: wgpu::BufferSize::new(size_of::<SurfaceFlagsGpu>() as u64),
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("mp_renderer_gpu world pipeline layout"),
            bind_group_layouts: &[
                Some(&globals_layout),
                Some(&texture_layout),
                Some(&flags_layout),
            ],
            immediate_size: 0,
        });

        let globals_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("mp_renderer_gpu world globals uniform"),
            size: size_of::<[f32; 16]>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let identity: [f32; 16] = [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        gpu.queue()
            .write_buffer(&globals_buffer, 0, bytemuck::cast_slice(&identity));

        let globals_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("mp_renderer_gpu world globals bind group"),
            layout: &globals_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals_buffer.as_entire_binding(),
            }],
        });

        let flags_capacity = 1;
        let flags_buffer = create_flags_buffer(device, flags_capacity);
        let flags_bind_group = create_flags_bind_group(device, &flags_layout, &flags_buffer);

        Pipeline3d {
            shader,
            pipeline_layout,
            texture_layout,
            pipelines: HashMap::new(),
            surface_format: gpu.surface_format(),
            globals_buffer,
            globals_bind_group,
            flags_layout,
            flags_buffer,
            flags_bind_group,
            flags_capacity,
            depth: DepthTexture::new(gpu, width, height),
            warned: [false; Warned::COUNT],
        }
    }

    /// Recreates the depth texture on a target resize.
    pub fn resize(&mut self, gpu: &Gpu, width: u32, height: u32) {
        self.depth.resize(gpu, width, height);
    }

    /// Writes the clip matrix `correction * projection * model` into the
    /// globals uniform. The caller feeds `viewParms_t.world.modelMatrix` and
    /// `viewParms_t.projectionMatrix`.
    pub fn set_view(&self, gpu: &Gpu, model: &[f32; 16], projection: &[f32; 16]) {
        let clip = world_clip_matrix(model, projection);
        gpu.queue()
            .write_buffer(&self.globals_buffer, 0, bytemuck::cast_slice(&clip));
    }

    /// Draws the sorted world draw-surf list. Each `SurfaceGeometry::World`
    /// entry resolves its shader through `R_DecomposeSort`, binds its diffuse
    /// and lightmap textures, and draws its indexed range. Non-world entries
    /// are counted and skipped.
    ///
    /// The pass clears the depth buffer to 1.0 per view but loads the color
    /// target, because `Gpu::begin_frame` already cleared color for the frame.
    /// A later 2D pass or a second scene draws over the world.
    pub fn draw(
        &mut self,
        gpu: &Gpu,
        target: &TextureView,
        draw_surfs: &[DrawSurf<SurfaceGeometry>],
        geometry: &WorldGeometry,
        assets: &RenderAssets,
        gpu_images: &GpuImages,
    ) -> WorldStats {
        let mut stats = WorldStats::default();
        let items = self.collect_items(draw_surfs, geometry, assets, &mut stats);

        if items.is_empty() {
            return stats;
        }

        // Every pipeline the batch uses must exist before the pass borrows
        // `self` immutably.
        for item in &items {
            self.ensure_pipeline(gpu, item.key);
        }

        self.reserve_flags(gpu, items.len());
        self.write_flags(gpu, &items);

        let bind_groups: Vec<wgpu::BindGroup> = items
            .iter()
            .map(|item| {
                gpu_images.world_bind_group(gpu, &self.texture_layout, item.diffuse, item.lightmap)
            })
            .collect();

        let mut encoder = gpu
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("mp_renderer_gpu world encoder"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("mp_renderer_gpu world pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        // `Gpu::begin_frame` already cleared the color target.
                        // A second scene in the same frame, for example MP
                        // cgame's `CG_Draw3DModel`, must draw over the first,
                        // so the world pass loads the color target, it does not
                        // clear it. `RB_BeginDrawingView` clears color only
                        // under `r_fastsky`.
                        // Source: oracle/codemp/renderer/tr_scene.cpp:823-826
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            pass.set_bind_group(0, &self.globals_bind_group, &[]);
            pass.set_vertex_buffer(0, geometry.vertex_buffer.slice(..));
            pass.set_index_buffer(geometry.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            for (draw_index, item) in items.iter().enumerate() {
                let pipeline = self
                    .pipelines
                    .get(&item.key)
                    .expect("world pipeline was created for every item's key above");
                let offset = (draw_index as u64 * SURFACE_FLAGS_STRIDE) as u32;

                pass.set_pipeline(pipeline);
                pass.set_bind_group(1, &bind_groups[draw_index], &[]);
                pass.set_bind_group(2, &self.flags_bind_group, &[offset]);
                pass.draw_indexed(
                    item.range.first_index..item.range.first_index + item.range.index_count,
                    item.range.base_vertex,
                    0..1,
                );

                stats.surfaces_drawn += 1;
                stats.draw_calls += 1;
                if item.has_lightmap {
                    stats.lightmapped += 1;
                }
            }
        }
        gpu.queue().submit(std::iter::once(encoder.finish()));

        stats
    }

    /// Resolves every world draw surf into a [`WorldDrawItem`], counting the
    /// non-world and empty entries into `stats`.
    fn collect_items(
        &mut self,
        draw_surfs: &[DrawSurf<SurfaceGeometry>],
        geometry: &WorldGeometry,
        assets: &RenderAssets,
        stats: &mut WorldStats,
    ) -> Vec<WorldDrawItem> {
        let mut items: Vec<WorldDrawItem> = Vec::new();

        for surf in draw_surfs {
            let world_ref = match surf.surface {
                SurfaceGeometry::World(world_ref) => world_ref,
                _ => {
                    stats.skipped_non_world += 1;
                    continue;
                }
            };

            let index = world_ref_index(world_ref);
            let range = geometry.range(index);
            if range.index_count == 0 {
                stats.empty_surfaces += 1;
                continue;
            }

            let (_entity_num, shader_handle, _fog_num, _dlight_map) =
                R_DecomposeSort(surf.sort, &assets.sorted_shaders);

            let (key, diffuse, lightmap, has_lightmap) =
                self.resolve_surface(shader_handle, assets);
            items.push(WorldDrawItem {
                range,
                key,
                diffuse,
                lightmap,
                has_lightmap,
            });
        }

        items
    }

    /// Resolves one surface's shader into a pipeline key, a diffuse image, and
    /// a lightmap image. A shader with no stages draws opaque with the default
    /// depth state. The lightmap comes from `RenderAssets::lightmaps` indexed by
    /// the shader's style-0 lightmap index. A negative index means no lightmap,
    /// so the surface shades by vertex color.
    fn resolve_surface(
        &mut self,
        shader_handle: ShaderHandle,
        assets: &RenderAssets,
    ) -> (PipelineKey, Option<ImageHandle>, Option<ImageHandle>, bool) {
        let Some(shader) = assets.shaders.get(shader_handle) else {
            return (default_pipeline_key(), None, None, false);
        };

        self.warn_features(shader);

        // The pipeline state comes from the first active stage's GLS bits, or
        // opaque defaults when the shader has no active stage.
        let first_active = shader.stages.iter().find(|stage| stage.active);
        let key = match first_active {
            Some(stage) => PipelineKey {
                blend: blend_state_from_gls(stage.state_bits),
                depth_equal: (stage.state_bits & GLS_DEPTHFUNC_EQUAL as u32) != 0,
                depth_write: (stage.state_bits & GLS_DEPTHMASK_TRUE as u32) != 0,
            },
            None => default_pipeline_key(),
        };

        // The diffuse texture is the first active non-lightmap stage's image,
        // or the first active stage's image when every stage is a lightmap.
        let diffuse = shader
            .stages
            .iter()
            .filter(|stage| stage.active)
            .find(|stage| !stage.bundle[0].is_lightmap && stage.bundle[0].image.is_some())
            .or_else(|| shader.stages.iter().find(|stage| stage.active))
            .and_then(|stage| stage.bundle[0].image);

        let lightmap_index = shader.lightmap_index[0];
        let lightmap = if lightmap_index >= 0 {
            assets.lightmaps.get(lightmap_index as usize).copied()
        } else {
            None
        };
        let has_lightmap = lightmap.is_some();

        (key, diffuse, lightmap, has_lightmap)
    }

    /// Counts and warns once for each out-of-scope shader feature a surface
    /// carries.
    fn warn_features(&mut self, shader: &ShaderAsset) {
        if shader.fog_parms.is_some() {
            self.warn_once(Warned::Fog);
        }

        let active: Vec<_> = shader.stages.iter().filter(|stage| stage.active).collect();
        if active.len() > 2 {
            self.warn_once(Warned::MultiStage);
        }

        for stage in &active {
            let bundle = &stage.bundle[0];
            if !bundle.tex_mods.is_empty() {
                self.warn_once(Warned::TcMod);
            }
            if bundle.num_image_animations > 0 {
                self.warn_once(Warned::AnimMap);
            }
            if stage.rgb_gen == colorGen_t::CGEN_WAVEFORM {
                self.warn_once(Warned::RgbGenWave);
            }
        }
    }

    /// Builds and caches the pipeline for `key` if it is not already there.
    fn ensure_pipeline(&mut self, gpu: &Gpu, key: PipelineKey) {
        if self.pipelines.contains_key(&key) {
            return;
        }
        let depth_compare = if key.depth_equal {
            wgpu::CompareFunction::Equal
        } else {
            wgpu::CompareFunction::LessEqual
        };

        let pipeline = gpu
            .device()
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("mp_renderer_gpu world pipeline"),
                layout: Some(&self.pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &self.shader,
                    entry_point: Some("vs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[Some(wgpu::VertexBufferLayout {
                        array_stride: size_of::<WorldVertex>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &VERTEX_ATTRIBUTES,
                    })],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    // Culling is off for this wave: the frontend has already
                    // culled surfaces, and per-shader cull sidedness lands with
                    // a later wave.
                    cull_mode: None,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: Some(key.depth_write),
                    depth_compare: Some(depth_compare),
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &self.shader,
                    entry_point: Some("fs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: self.surface_format,
                        blend: Some(key.blend),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: None,
            });
        self.pipelines.insert(key, pipeline);
    }

    /// Grows the per-surface flags buffer (and its bind group) when `needed`
    /// exceeds capacity.
    fn reserve_flags(&mut self, gpu: &Gpu, needed: usize) {
        if needed <= self.flags_capacity {
            return;
        }
        let mut capacity = self.flags_capacity.max(1);
        while capacity < needed {
            capacity *= 2;
        }
        self.flags_buffer = create_flags_buffer(gpu.device(), capacity);
        self.flags_bind_group =
            create_flags_bind_group(gpu.device(), &self.flags_layout, &self.flags_buffer);
        self.flags_capacity = capacity;
    }

    /// Writes one `has_lightmap` flag per draw item into the flags buffer, each
    /// at its own stride slot so the dynamic offset lands on it.
    fn write_flags(&self, gpu: &Gpu, items: &[WorldDrawItem]) {
        let mut bytes = vec![0u8; items.len() * SURFACE_FLAGS_STRIDE as usize];
        for (draw_index, item) in items.iter().enumerate() {
            let flags = SurfaceFlagsGpu {
                has_lightmap: item.has_lightmap as u32,
                _pad: [0; 3],
            };
            let offset = draw_index * SURFACE_FLAGS_STRIDE as usize;
            let src = bytemuck::bytes_of(&flags);
            bytes[offset..offset + src.len()].copy_from_slice(src);
        }
        gpu.queue().write_buffer(&self.flags_buffer, 0, &bytes);
    }

    /// Logs an out-of-scope feature the first time it is seen.
    fn warn_once(&mut self, kind: Warned) {
        let slot = kind.slot();
        if self.warned[slot] {
            return;
        }
        self.warned[slot] = true;
        eprintln!("mp_renderer_gpu: pipeline3d {}", kind.describe());
    }
}

/// The `WorldSurfaceRef` flat surface index, regardless of kind.
fn world_ref_index(world_ref: WorldSurfaceRef) -> u32 {
    match world_ref {
        WorldSurfaceRef::Skip(index)
        | WorldSurfaceRef::Face(index)
        | WorldSurfaceRef::Grid(index)
        | WorldSurfaceRef::Triangles(index)
        | WorldSurfaceRef::Flare(index) => index,
    }
}

/// The opaque, depth-writing, less-equal state a stage-less shader draws with.
fn default_pipeline_key() -> PipelineKey {
    PipelineKey {
        blend: blend_state_from_gls(0),
        depth_equal: false,
        depth_write: true,
    }
}

fn texture_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

fn sampler_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        count: None,
    }
}

fn create_buffer(
    device: &wgpu::Device,
    label: &str,
    contents: &[u8],
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: contents.len() as u64,
        usage: usage | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: true,
    });
    buffer
        .slice(..)
        .get_mapped_range_mut()
        .expect("a freshly mapped buffer maps its full range")
        .copy_from_slice(contents);
    buffer.unmap();
    buffer
}

fn create_flags_buffer(device: &wgpu::Device, capacity: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("mp_renderer_gpu world flags buffer"),
        size: capacity as u64 * SURFACE_FLAGS_STRIDE,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn create_flags_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("mp_renderer_gpu world flags bind group"),
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer,
                offset: 0,
                size: wgpu::BufferSize::new(size_of::<SurfaceFlagsGpu>() as u64),
            }),
        }],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use mp_engine_qcommon::qfiles::draw_vert_t::MAXLIGHTMAPS;

    fn sample_draw_vert() -> drawVert_t {
        drawVert_t {
            xyz: [1.0, 2.0, 3.0],
            st: [0.25, 0.5],
            lightmap: [[0.1, 0.2], [9.0, 9.0], [9.0, 9.0], [9.0, 9.0]],
            normal: [0.0, 0.0, 1.0],
            color: [[10, 20, 30, 40], [1, 1, 1, 1], [1, 1, 1, 1], [1, 1, 1, 1]],
        }
    }

    fn sample_face_vertex() -> FaceVertex {
        FaceVertex {
            xyz: [1.0, 2.0, 3.0],
            st: [0.25, 0.5],
            lightmap: [[0.1, 0.2], [9.0, 9.0], [9.0, 9.0], [9.0, 9.0]],
            color: [[10, 20, 30, 40], [1, 1, 1, 1], [1, 1, 1, 1], [1, 1, 1, 1]],
        }
    }

    // vertex convergence

    #[test]
    fn face_and_draw_vert_converge_to_the_same_world_vertex() {
        let from_face = WorldVertex::from_face_vertex(&sample_face_vertex());
        let from_draw = WorldVertex::from_draw_vert(&sample_draw_vert());
        // The two shapes carry the same xyz, st, style-0 lightmap and style-0
        // color, so they converge exactly. The `drawVert_t` normal is dropped.
        assert_eq!(from_face, from_draw);
    }

    #[test]
    fn world_vertex_keeps_style_zero_only() {
        let vertex = WorldVertex::from_draw_vert(&sample_draw_vert());
        assert_eq!(vertex.position, [1.0, 2.0, 3.0]);
        assert_eq!(vertex.st, [0.25, 0.5]);
        assert_eq!(vertex.lightmap_st, [0.1, 0.2]);
        assert_eq!(vertex.color, [10, 20, 30, 40]);
        assert_eq!(MAXLIGHTMAPS, 4);
    }

    // grid index emission

    #[test]
    fn grid_indices_emit_two_triangles_per_cell() {
        // A 2x2 lattice is one cell — two triangles, six indices.
        let indices = grid_indices(2, 2);
        assert_eq!(indices, vec![0, 2, 1, 1, 2, 3]);
    }

    #[test]
    fn grid_indices_row_major_for_a_three_by_two() {
        // A 3x2 lattice is two cells across one row.
        let indices = grid_indices(3, 2);
        assert_eq!(
            indices,
            vec![
                0, 3, 1, 1, 3, 4, // cell (0,0)
                1, 4, 2, 2, 4, 5, // cell (1,0)
            ]
        );
        assert_eq!(indices.len(), 2 * 6);
    }

    #[test]
    fn a_degenerate_grid_emits_no_triangle() {
        assert!(grid_indices(1, 5).is_empty());
        assert!(grid_indices(5, 1).is_empty());
        assert!(grid_indices(0, 0).is_empty());
    }

    // z remap

    #[test]
    fn depth_correction_maps_the_near_plane_to_zero_and_far_to_one() {
        let correction = depth_correction();
        // A GL near-plane clip point sits at NDC z -1, so clip (0, 0, -1, 1).
        let near = mat4_mul_vec(&correction, [0.0, 0.0, -1.0, 1.0]);
        assert!((near[2] / near[3]).abs() < 1e-6, "near z = {}", near[2]);
        // A GL far-plane clip point sits at NDC z +1, so clip (0, 0, 1, 1).
        let far = mat4_mul_vec(&correction, [0.0, 0.0, 1.0, 1.0]);
        assert!((far[2] / far[3] - 1.0).abs() < 1e-6, "far z = {}", far[2]);
    }

    #[test]
    fn world_clip_matrix_remaps_a_gl_frustum_to_zero_one() {
        // A standard GL perspective frustum, column-major, near 4, far 100.
        let z_near = 4.0f32;
        let z_far = 100.0f32;
        let depth = z_far - z_near;
        let mut projection = [0.0f32; 16];
        projection[0] = 1.0;
        projection[5] = 1.0;
        projection[10] = -(z_far + z_near) / depth;
        projection[14] = -2.0 * z_far * z_near / depth;
        projection[11] = -1.0;

        let identity: [f32; 16] = [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        let clip = world_clip_matrix(&identity, &projection);

        // An eye-space point on the near plane (z = -near) lands at wgpu z 0.
        let near = mat4_mul_vec(&clip, [0.0, 0.0, -z_near, 1.0]);
        assert!(
            (near[2] / near[3]).abs() < 1e-4,
            "near z = {}",
            near[2] / near[3]
        );
        // An eye-space point on the far plane (z = -far) lands at wgpu z 1.
        let far = mat4_mul_vec(&clip, [0.0, 0.0, -z_far, 1.0]);
        assert!(
            (far[2] / far[3] - 1.0).abs() < 1e-4,
            "far z = {}",
            far[2] / far[3]
        );
    }
}
