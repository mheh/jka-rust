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
//! and a per-pass flags block (group 2, dynamic offset) drive the `world.wgsl`
//! shader. `RB_IterateStagesGeneric` draws one pass per active stage over the
//! same geometry, so this module draws one indexed pass per active stage per
//! surface, in stage order. Each pass carries its own image, its own blend and
//! depth state from `pStage->stateBits`, and its own per-vertex colours and
//! texcoords.
//!
//! A stage whose bundle needs no per-frame vertex work (no tcMods, no waveform
//! colour) draws from the static world buffer. A stage with dynamic texcoords
//! or colours gets its per-vertex data evaluated on the CPU each frame through
//! the shared `stage2d` evaluators and written to a per-frame dynamic buffer.
//!
//! Out of this wave: fog (warned) and the sky chain (skipped and warned). tcMod,
//! animMap, rgbGen/alphaGen waves, and multi-stage shaders are real here.
//!
//! DEC-37 ruling 2/3: this is render-thread-owned state. Nothing here is
//! reachable from a trap query.
//!
//! Source: `oracle/codemp/renderer/tr_shade.cpp:1953-2231` (`RB_IterateStagesGeneric`)

use core::f64::consts::PI;
use std::collections::HashMap;
use std::mem::size_of;

use bytemuck::{Pod, Zeroable};
use mp_engine_ghoul2::ghoul2_system::Ghoul2System;
use mp_engine_qcommon::qfiles::draw_vert_t::drawVert_t;
use mp_engine_qcommon::qfiles::md3_limits::MD3_XYZ_SCALE;
use mp_engine_qcommon::qfiles::md3_surface_t::md3Surface_t;
use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::common::mp::cgame::poly_vert_t::polyVert_t;
use mp_qshared::common::mp::cgame::ref_entity_type_t::refEntityType_t;
use mp_qshared::common::mp::cgame::tr_types::{
    RF_ALPHA_DEPTH, RF_DEPTHHACK, RF_DISINTEGRATE1, RF_DISINTEGRATE2, RF_DISTORTION,
    RF_FORCE_ENT_ALPHA, RF_NODEPTH, RF_RGB_TINT, RF_VOLUMETRIC,
};
use mp_qshared::shared::mdxaBone_t;
use mp_qshared::shared::q_math::{
    _VectorMA as VectorMA, _VectorScale as VectorScale, _VectorSubtract as VectorSubtract,
    vec3_origin, CrossProduct, VectorNormalize,
};
use mp_qshared::shared::vec3_t;
use mp_renderer::render_state::frame_state::FrameState;
use mp_renderer::render_state::image_asset::ImageHandle;
use mp_renderer::render_state::placeholders::{RefEntity, SkyParms, WorldAsset, FUNCTABLE_SIZE};
use mp_renderer::render_state::render_assets::RenderAssets;
use mp_renderer::render_state::render_cvar_snapshot::RenderCvarSnapshot;
use mp_renderer::render_state::shader_asset::ShaderAsset;
use mp_renderer::render_state::shader_stage::ShaderStage;
use mp_renderer::render_state::texture_bundle::TextureBundle;
use mp_renderer::tr_bsp::{FaceVertex, SurfaceData};
use mp_renderer::tr_curve::GridMesh;
use mp_renderer::tr_local::acff_t::acff_t;
use mp_renderer::tr_local::alpha_gen_t::alphaGen_t;
use mp_renderer::tr_local::color_gen_t::colorGen_t;
use mp_renderer::tr_local::fog_t::fog_t;
use mp_renderer::tr_local::orientationr_t::orientationr_t;
use mp_renderer::tr_local::tex_coord_gen_t::texCoordGen_t;
use mp_renderer::tr_local::tr_ref_entity_t::trRefEntity_t;
use mp_renderer::tr_local::view_parms_t::viewParms_t;
use mp_renderer::tr_main::{
    DrawSurf, G2SurfaceRef, Md3SurfaceRef, R_DecomposeSort, R_RotateForEntity, SurfaceGeometry,
    TrMainScratch, WorldSurfaceRef,
};
use mp_renderer::tr_model::frontend::mdxm_view_of;
use mp_renderer::tr_model::render_models::RenderModels;
use mp_renderer::tr_noise::NoiseState;
use mp_renderer::tr_public::ref_flags::{RDF_NOWORLDMODEL, RDF_SKYBOXPORTAL};
use mp_renderer::tr_shade::RB_FogPass;
use mp_renderer::tr_shade_calc::{
    myftol, RB_CalcDiffuseColor, RB_CalcDiffuseEntityColor, RB_CalcDisintegrateColors,
    RB_CalcDisintegrateVertDeform, RB_CalcModulateAlphasByFog, RB_CalcModulateColorsByFog,
    RB_CalcModulateRGBAsByFog,
};
use mp_renderer::tr_shader::{
    FogPass, GLS_ATEST_GE_80, GLS_ATEST_GE_C0, GLS_ATEST_GT_0, GLS_ATEST_LT_80,
    GLS_DEPTHFUNC_EQUAL, GLS_DEPTHMASK_TRUE, GLS_DSTBLEND_ONE_MINUS_SRC_ALPHA,
    GLS_SRCBLEND_SRC_ALPHA, GL_MODULATE,
};
use mp_renderer::tr_sky::{RB_StageIteratorSky, SkyBoxFace, SkyState, HALF_SKY_SUBDIVISIONS};
use mp_renderer::tr_surface::LodErrorForVolume;
use wgpu::{BlendState, RenderPipeline, TextureView};

use crate::blend::{blend_state_from_gls, OPAQUE};
use crate::gpu::Gpu;
use crate::gpu_images::GpuImages;
use native_math::qmath::Q_rsqrt;

use crate::stage2d::{
    apply_alpha_gen, apply_tex_mods, stage_colors_into, stage_image, Stage2dWarnings, StageTime,
    IDENTITY_LIGHT,
};

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

    /// This returns the vertex position in the surface's own space. The
    /// differential vertex golden reads it to serialize the ghoul2 stream.
    pub fn position(&self) -> [f32; 3] {
        self.position
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
    /// The vertex count this surface owns, so the CPU evaluators can slice its
    /// own block out of `WorldGeometry::cpu_vertices`.
    pub vertex_count: u32,
}

impl SurfaceRange {
    /// The empty range a `Skip` or `Flare` surface gets.
    const EMPTY: SurfaceRange = SurfaceRange {
        base_vertex: 0,
        first_index: 0,
        index_count: 0,
        vertex_count: 0,
    };
}

/// The world's uploaded geometry: one concatenated vertex buffer, one index
/// buffer, and a per-surface range vector parallel to `WorldAsset::surfaces`.
pub struct WorldGeometry {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    ranges: Vec<SurfaceRange>,
    /// The same vertices the static buffer holds, kept on the CPU so a stage
    /// with dynamic texcoords or colours can re-evaluate its own block per
    /// frame.
    cpu_vertices: Vec<WorldVertex>,
    /// The same indices the static buffer holds, kept on the CPU so a sky-shader
    /// surface can hand its own triangles to the sky-box projection. Each range
    /// slice holds the surface's own 0-based triangle indices.
    cpu_indices: Vec<u32>,
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
            cpu_vertices: vertices,
            cpu_indices: indices,
        }
    }

    /// The world-less geometry a `RDF_NOWORLDMODEL` scene draws against: valid
    /// but empty buffers and no surface ranges. Entity, poly, and sprite
    /// surfaces all build their own per-frame vertices, so a scene with no BSP
    /// never reads these buffers. It still needs the handles to bind.
    pub fn empty(gpu: &Gpu) -> WorldGeometry {
        // wgpu rejects a zero-size buffer, so both buffers hold one zeroed
        // element, the same fallback `upload` makes for an empty world.
        let vertex_fallback = [WorldVertex::zeroed()];
        let index_fallback = [0u32];
        let vertex_buffer = create_buffer(
            gpu.device(),
            "mp_renderer_gpu empty world vertex buffer",
            bytemuck::cast_slice(&vertex_fallback),
            wgpu::BufferUsages::VERTEX,
        );
        let index_buffer = create_buffer(
            gpu.device(),
            "mp_renderer_gpu empty world index buffer",
            bytemuck::cast_slice(&index_fallback),
            wgpu::BufferUsages::INDEX,
        );

        WorldGeometry {
            vertex_buffer,
            index_buffer,
            ranges: Vec::new(),
            cpu_vertices: Vec::new(),
            cpu_indices: Vec::new(),
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
        let vertex_count = vertices.len() as u32 - base_vertex as u32;
        ranges.push(SurfaceRange {
            base_vertex,
            first_index,
            index_count,
            vertex_count,
        });
    }

    (vertices, indices, ranges)
}

/// The row-major triangle indices for a `width` by `height` grid lattice, two
/// triangles per cell. The indices are 0-based within the grid's own vertex
/// block. A grid narrower or shorter than two verts has no cell, so no
/// triangle.
///
/// This is the "keep every row and column" fast path of `RB_SurfaceGrid`. When
/// the surface LOD keeps the full lattice, [`grid_lod_indices`] emits the
/// identical index set (its `widthTable`/`heightTable` become `0..width`/
/// `0..height`), so the grid uploads these static indices once at level load
/// and skips the per-frame subsample. The oracle's `SHADER_MAX_VERTEXES` split
/// (the multi-pass flush at `:1622-1638`) has no wgpu target: the shared world
/// buffer holds the whole map, so one draw covers any patch.
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:1572` (indices at `:1736-1755`)
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

/// The per-frame LOD-subsampled triangle indices for a bezier-patch grid,
/// 0-based within the grid's own full vertex block. `RB_SurfaceGrid` builds
/// `widthTable`/`heightTable` by keeping the two edge rows/columns plus every
/// interior row/column whose stored LOD error is within `lodError`, then walks
/// the reduced lattice. The vertex it fetches for reduced cell corner `(h, w)`
/// is `verts + heightTable[h] * width + widthTable[w]`, so the emitted index is
/// that same offset into the full vertex block (the shared world buffer already
/// holds every vert). The two triangles per cell keep the oracle's tristrip
/// winding (`v2,v3,v1` then `v1,v3,v4`), which is the same winding
/// [`grid_indices`] uses.
///
/// When the LOD keeps every row and column, `widthTable`/`heightTable` become
/// the dense `0..width`/`0..height`, so this returns the exact index set
/// [`grid_indices`] builds. The caller compares the reduced lattice size
/// against the full size and keeps the static path in that case.
///
/// The oracle's `SHADER_MAX_VERTEXES` multi-pass split is dropped: the shared
/// world buffer holds the whole map, so one draw covers any patch.
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:1594-1638` (LOD tables),
/// `:1740-1755` (index winding)
fn grid_lod_indices(
    grid: &GridMesh,
    ori: &orientationr_t,
    view: &viewParms_t,
    lod_curve_error: f32,
) -> Vec<u32> {
    let width = grid.width;
    let height = grid.height;
    if width < 2 || height < 2 {
        return Vec::new();
    }

    let lod_error =
        LodErrorForVolume(grid.lod_origin, grid.lod_radius, ori, view, lod_curve_error);

    // Keep the two edge columns, plus every interior column within tolerance.
    let mut width_table: Vec<i32> = Vec::with_capacity(width as usize);
    width_table.push(0);
    for i in 1..width - 1 {
        if grid.width_lod_error[i as usize] <= lod_error {
            width_table.push(i);
        }
    }
    width_table.push(width - 1);

    // Keep the two edge rows, plus every interior row within tolerance.
    let mut height_table: Vec<i32> = Vec::with_capacity(height as usize);
    height_table.push(0);
    for i in 1..height - 1 {
        if grid.height_lod_error[i as usize] <= lod_error {
            height_table.push(i);
        }
    }
    height_table.push(height - 1);

    let lod_width = width_table.len();
    let lod_height = height_table.len();

    let mut indices: Vec<u32> = Vec::with_capacity((lod_width - 1) * (lod_height - 1) * 6);
    for h in 0..lod_height - 1 {
        for w in 0..lod_width - 1 {
            // Corner offsets in the grid's own full vertex block, the same
            // `heightTable[..] * width + widthTable[..]` fetch the oracle uses.
            let top_left = (height_table[h] * width + width_table[w]) as u32;
            let top_right = (height_table[h] * width + width_table[w + 1]) as u32;
            let bottom_left = (height_table[h + 1] * width + width_table[w]) as u32;
            let bottom_right = (height_table[h + 1] * width + width_table[w + 1]) as u32;

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

/// The per-entity clip-matrix stride for the dynamic-offset globals buffer.
/// wgpu requires a dynamic offset that is a multiple of
/// `min_uniform_buffer_offset_alignment`, at most 256 on every backend, so 256
/// is always legal. One aligned slot holds one `[f32; 16]` clip matrix.
const GLOBALS_STRIDE: u64 = 256;

/// The clip-matrix binding size, one `[f32; 16]`.
const CLIP_MATRIX_SIZE: u64 = size_of::<[f32; 16]>() as u64;

/// Raven `MAX_ENTITIES` — the per-frame ref-entity bound. The non-`_XBOX`
/// build selects 2048; `_XBOX` selects 1024. We build the 2048 target.
///
/// Source: `oracle/codemp/cgame/tr_types.h:9-12`
const MAX_ENTITIES: i32 = 2048;

/// The world entity number the frontend tags every world (non-inline-model)
/// surface with (`R_AddWorldSurfaces` sets `tr.currentEntityNum = TR_WORLDENT`).
/// The world slot is slot 0 of the globals buffer, so a draw surf decoded to
/// this number uses the view matrix, not a per-entity matrix. `MAX_ENTITIES - 1`
/// reserves the last slot for the world.
///
/// Source: `oracle/codemp/cgame/tr_types.h:15`
const TR_WORLDENT: i32 = MAX_ENTITIES - 1;

/// The per-pass uniform the fragment shader reads through group 2. `mode`
/// picks the single-texture or two-texture path, `tex_from_lightmap` selects
/// the lightmap texcoord for a single-texture lightmap stage, and `alpha_func`
/// tells the shader which `GLS_ATEST` compare to discard by. `pbr_lit` marks a
/// stage the PBR backend shades. The fourth word also rounds the write size to
/// 16 bytes.
///
/// The faithful `world.wgsl` declares only the first three fields, so it ignores
/// `pbr_lit` and a default frame stays byte-for-byte unchanged. The PBR
/// `world_pbr.wgsl` reads `pbr_lit` to route the stage.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct SurfaceFlagsGpu {
    mode: u32,
    tex_from_lightmap: u32,
    alpha_func: u32,
    pbr_lit: u32,
}

/// The single-texture pass: sample bundle 0 and multiply by the per-vertex
/// colour (`RB_IterateStagesGeneric`'s common `R_DrawElements` arm).
const MODE_SINGLE: u32 = 0;
/// The two-texture pass: sample bundle 0 times the lightmap
/// (`DrawMultitextured` under `GL_MODULATE`).
///
/// `CollapseMultitexture` in `tr_shader.rs` now lands the collapse, so
/// `ShaderAsset::multitexture_env` carries `GL_MODULATE`. This two-texture path
/// is live when bundle 1 is a lightmap. A lightmapped world surface can still
/// draw its lightmap as its own single-texture stage through the
/// `tex_from_lightmap` flag.
const MODE_MULTITEXTURE: u32 = 1;

/// The shader features the world backend cannot render yet, tracked so each one
/// logs once per process rather than once per surface.
#[derive(Clone, Copy, Debug)]
enum Warned {
    TcGen,
    MultitexEnv,
    SurfaceSprite,
    Glow,
    VideoMap,
    /// A fog pass was due but the fog image is not registered.
    FogImageMissing,
    /// An `SF_ENTITY` draw surf whose `reType` builds no geometry yet.
    EntitySurface,
    /// An `RF_DISTORTION` entity, which needs the screen-capture refraction
    /// pass.
    Distortion,
}

impl Warned {
    const COUNT: usize = 8;

    fn slot(self) -> usize {
        match self {
            Warned::TcGen => 0,
            Warned::MultitexEnv => 1,
            Warned::SurfaceSprite => 2,
            Warned::Glow => 3,
            Warned::VideoMap => 4,
            Warned::FogImageMissing => 5,
            Warned::EntitySurface => 6,
            Warned::Distortion => 7,
        }
    }

    fn describe(self) -> &'static str {
        match self {
            Warned::TcGen => "reads base texcoords for an unsupported tcGen on a world stage",
            Warned::MultitexEnv => {
                "draws bundle 0 only for a non-modulate multitexture world shader"
            }
            Warned::SurfaceSprite => "skips a surface-sprite world stage, drawn in a later wave",
            Warned::Glow => "draws a glow world stage as a plain stage",
            Warned::VideoMap => "draws a videoMap world stage as a plain stage — no cinematic yet",
            Warned::FogImageMissing => "skips a fog pass because the fog image is not registered",
            Warned::EntitySurface => "skips a generated entity surface whose reType is not ported",
            Warned::Distortion => "draws an RF_DISTORTION entity plain - no screen capture yet",
        }
    }
}

/// What one [`Pipeline3d::draw`] call did.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WorldStats {
    /// World surfaces that drew at least one stage pass. Inline brush-model
    /// entity surfaces count here too, since they draw through the same world
    /// path.
    pub surfaces_drawn: u32,
    /// The subset of `surfaces_drawn` that belong to a real entity (an inline
    /// brush model), not the world itself. A surface counts here when its sort
    /// key decodes to an entity number other than the world entity.
    pub entity_surfaces_drawn: u32,
    /// `draw_indexed` calls issued — one per active stage per surface.
    pub draw_calls: u32,
    /// Stage passes that bound the lightmap through the two-texture path.
    pub lightmapped: u32,
    /// Non-world draw-surf entries skipped (entity models, polys, and so on).
    pub skipped_non_world: u32,
    /// World surfaces whose range was empty (`Skip`/`Flare`), so nothing drew.
    pub empty_surfaces: u32,
    /// Scene polygons (`SF_POLY`) that drew at least one stage pass. The census
    /// mark path.
    pub poly_surfaces_drawn: u32,
    /// Sky-shader surfaces that drew their sky box, clouds, or both. The oracle
    /// forks a sky shader into `RB_StageIteratorSky`. A surface counts here when
    /// that chain drew at least one face or cloud pass.
    pub sky_surfaces_drawn: u32,
    /// MD3 (`MOD_MESH`) entity surfaces that drew at least one stage pass.
    pub md3_surfaces_drawn: u32,
    /// MD3 draw surfs the decode dropped (a bad model handle or a purged model).
    /// This stays separate from `skipped_non_world` so the two causes read apart.
    pub md3_decode_failed: u32,
    /// Ghoul2 (`MOD_MDXM`) entity surfaces that drew at least one stage pass.
    pub ghoul2_surfaces_drawn: u32,
    /// Ghoul2 draw surfs the decode dropped (a stale bone-cache handle, a null
    /// mdxm block, or an empty surface). Separate from `skipped_non_world` so the
    /// causes read apart.
    pub ghoul2_decode_failed: u32,
    /// Fog passes drawn — one extra pass per fogged surface whose shader
    /// declares a `fogPass` (`RB_FogPass` at the tail of `RB_StageIteratorGeneric`).
    pub fog_passes_drawn: u32,
}

/// The shader backend the world pass draws one frame with (DEC-37 ruling 5).
/// The draw derives it once per call from `RenderCvarSnapshot::pbr`, then reads
/// it at the two seams that pick per-draw state: the pipeline the draw binds,
/// and the texture bind group it builds. A `Faithful` frame runs the existing
/// path with no change. A `Pbr` frame runs the PBR pipeline and the
/// four-texture PBR bind group.
///
/// - `Faithful`: backend #1, the parity reference, `pbr == 0`.
/// - `Pbr`: backend #2, the PBR uber-shader, `pbr != 0`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BackendMode {
    Faithful,
    Pbr,
}

impl BackendMode {
    /// Reads the backend the snapshot selects. Zero is the retail default and
    /// the parity reference.
    fn from_snapshot(cvars: RenderCvarSnapshot) -> BackendMode {
        if cvars.pbr != 0 {
            BackendMode::Pbr
        } else {
            BackendMode::Faithful
        }
    }
}

/// The pipeline cache key: one pipeline per distinct blend state and depth
/// state. `BlendState` is `Hash`; the two depth choices are booleans.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct PipelineKey {
    blend: BlendState,
    depth_equal: bool,
    depth_write: bool,
}

/// The resolved state one stage pass draws with, collected before the render
/// pass so the pass borrows `self` immutably. `dynamic` selects the per-frame
/// dynamic vertex buffer over the static world buffer, and `base_vertex` then
/// indexes that buffer.
struct StageDrawItem {
    key: PipelineKey,
    diffuse: Option<ImageHandle>,
    lightmap: Option<ImageHandle>,
    mode: u32,
    tex_from_lightmap: bool,
    /// The `GLS_ATEST` compare code the shader discards by. See [`alpha_func_code`].
    alpha_func: u32,
    /// Whether this pass reads a lightmap texture, so `WorldStats::lightmapped`
    /// counts the real lightmapping path.
    reads_lightmap: bool,
    first_index: u32,
    index_count: u32,
    base_vertex: i32,
    dynamic: bool,
    /// Whether this pass reads its indices from the per-frame dynamic index
    /// buffer (MD3 entity surfaces) rather than the static world index buffer.
    index_dynamic: bool,
    /// The byte offset of this surface's clip matrix in the dynamic-offset
    /// globals buffer. Slot 0 (offset 0) is the world matrix.
    globals_offset: u32,
    /// Whether this pass draws under the far-plane depth range (`qglDepthRange(
    /// 1.0, 1.0)`), the sky-box and cloud state. The draw loop calls
    /// `set_viewport` with `min_depth = max_depth = 1.0` around the run of
    /// far-plane items and restores `0.0..1.0` after.
    ///
    /// Source: `oracle/codemp/renderer/tr_sky.cpp:814`
    depth_far: bool,
    /// The entity's depth-range hack, `backEnd`'s `depthRange` switch. The draw
    /// loop applies it the same way it applies `depth_far`.
    ///
    /// Source: `oracle/codemp/renderer/tr_backend.cpp:930-938,957-973`
    depth_range: DepthRange,
}

/// Raven's `depthRange` switch in `RB_RenderDrawSurfList`: the depth window a
/// surface's entity draws into.
///
/// - `Normal`: `qglDepthRange(0, 1)`, every ordinary surface.
/// - `Hack`: `qglDepthRange(0, 0.3)`, `RF_DEPTHHACK`, which keeps a view model
///   from poking into walls.
/// - `None`: `qglDepthRange(0, 0)`, `RF_NODEPTH`, for seeing through walls.
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:930-938,957-973`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DepthRange {
    Normal,
    Hack,
    None,
}

impl DepthRange {
    /// The entity's range. `RF_NODEPTH` wins, matching the oracle's `if`/`else
    /// if` order.
    fn resolve(entity: Option<&trRefEntity_t>) -> DepthRange {
        let Some(ent) = entity else {
            return DepthRange::Normal;
        };
        if ent.e.renderfx & RF_NODEPTH != 0 {
            DepthRange::None
        } else if ent.e.renderfx & RF_DEPTHHACK != 0 {
            DepthRange::Hack
        } else {
            DepthRange::Normal
        }
    }

    /// The `min_depth`, `max_depth` pair `set_viewport` takes.
    fn window(self) -> (f32, f32) {
        match self {
            DepthRange::Normal => (0.0, 1.0),
            DepthRange::Hack => (0.0, 0.3),
            DepthRange::None => (0.0, 0.0),
        }
    }
}

/// The fog inputs one surface's stages read: the resolved fog volume and the
/// two orientations `RB_CalcFogTexCoords` needs. The frontend tags a nonzero
/// fog number on a fogged surface, so a surface with fog number 0 gets `None`
/// and runs no fog work.
///
/// `ori` is the surface's model orientation (`backEnd.ori`), the world
/// orientation for a world surface or `R_RotateForEntity`'s result for an
/// entity. `view_ori` is the camera orientation (`backEnd.viewParms.ori`).
///
/// Source: `oracle/codemp/renderer/tr_shade_calc.cpp:983-1068` (`RB_CalcFogTexCoords`)
#[derive(Clone, Copy)]
struct SurfaceFog<'a> {
    fog: &'a fog_t,
    ori: &'a orientationr_t,
    view_ori: &'a orientationr_t,
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
    /// The PBR backend's shader, its four-texture group-1 layout, its pipeline
    /// layout, and its own per-state pipeline cache (DEC-37 ruling 5, backend
    /// #2). These sit beside the faithful resources and are built once. A
    /// `Faithful` frame never reads them.
    pbr_shader: wgpu::ShaderModule,
    pbr_texture_layout: wgpu::BindGroupLayout,
    pbr_pipeline_layout: wgpu::PipelineLayout,
    pbr_pipelines: HashMap<PipelineKey, RenderPipeline>,
    surface_format: wgpu::TextureFormat,
    globals_layout: wgpu::BindGroupLayout,
    globals_buffer: wgpu::Buffer,
    globals_bind_group: wgpu::BindGroup,
    /// The number of clip-matrix slots the globals buffer holds. It grows to
    /// cover the distinct entity numbers a scene uses and is reused across
    /// frames.
    globals_capacity: usize,
    flags_layout: wgpu::BindGroupLayout,
    flags_buffer: wgpu::Buffer,
    flags_bind_group: wgpu::BindGroup,
    flags_capacity: usize,
    /// The per-frame vertex buffer a dynamic stage writes its evaluated
    /// texcoords and colours into. It grows and is reused across frames.
    dynamic_buffer: wgpu::Buffer,
    dynamic_capacity: usize,
    /// The per-frame index buffer an MD3 entity surface writes its decoded
    /// triangle indices into. The static world index buffer holds only world
    /// surfaces, so MD3 indices need their own buffer. It grows and is reused
    /// across frames.
    dynamic_index_buffer: wgpu::Buffer,
    dynamic_index_capacity: usize,
    depth: DepthTexture,
    warned: [bool; Warned::COUNT],
    /// The dedup log the shared `stage2d` evaluators write their own rgbGen,
    /// alphaGen, and tcMod fallbacks into.
    stage_warnings: Stage2dWarnings,
    /// The test-only Ghoul2 vertex-stream capture sink. It stays `None` in normal
    /// use, so the Ghoul2 collect path pays only a null check. A test arms it
    /// through [`FrameExecutor::set_ghoul2_capture`], and the collect path pushes
    /// each decoded surface stream in draw-surf order.
    ///
    /// [`FrameExecutor::set_ghoul2_capture`]: crate::FrameExecutor::set_ghoul2_capture
    ghoul2_capture: Option<Vec<Ghoul2SurfaceCapture>>,
}

/// One captured Ghoul2 surface vertex stream: the decoded vertices, the triangle
/// indices, and the parallel bone-0 normals.
///
/// The differential vertex golden reads this to serialize a committed fixture.
pub type Ghoul2SurfaceCapture = (Vec<WorldVertex>, Vec<u32>, Vec<[f32; 4]>);

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
                    // The globals buffer holds one clip matrix per distinct
                    // entity number this scene draws, and the draw picks its
                    // matrix with a dynamic offset (slot 0 is the world).
                    has_dynamic_offset: true,
                    min_binding_size: wgpu::BufferSize::new(CLIP_MATRIX_SIZE),
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

        // The PBR backend (DEC-37 ruling 5). Its shader shares group 0 and group
        // 2 with the faithful path. Group 1 grows to the four-texture material
        // set: diffuse and lightmap in 0-3, normal and roughness in 4-7. The
        // stage A shader reads only 0-3, so the extra layout entries are unused
        // for now, which wgpu allows. Stage B reads the normal and roughness.
        let pbr_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("mp_renderer_gpu pbr world shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/world_pbr.wgsl").into()),
        });

        let pbr_texture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("mp_renderer_gpu pbr world texture layout"),
            entries: &[
                texture_entry(0),
                sampler_entry(1),
                texture_entry(2),
                sampler_entry(3),
                texture_entry(4),
                sampler_entry(5),
                texture_entry(6),
                sampler_entry(7),
            ],
        });

        let pbr_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("mp_renderer_gpu pbr world pipeline layout"),
            bind_group_layouts: &[
                Some(&globals_layout),
                Some(&pbr_texture_layout),
                Some(&flags_layout),
            ],
            immediate_size: 0,
        });

        // `draw` calls `write_globals` before the pass reads the buffer, and
        // `build_entity_slots` always fills slot 0 with the world matrix, so the
        // buffer needs no construction-time default.
        let globals_capacity = 1;
        let globals_buffer = create_globals_buffer(device, globals_capacity);

        let globals_bind_group =
            create_globals_bind_group(device, &globals_layout, &globals_buffer);

        let flags_capacity = 1;
        let flags_buffer = create_flags_buffer(device, flags_capacity);
        let flags_bind_group = create_flags_bind_group(device, &flags_layout, &flags_buffer);

        let dynamic_capacity = 1;
        let dynamic_buffer = create_dynamic_buffer(device, dynamic_capacity);

        let dynamic_index_capacity = 1;
        let dynamic_index_buffer = create_dynamic_index_buffer(device, dynamic_index_capacity);

        Pipeline3d {
            shader,
            pipeline_layout,
            texture_layout,
            pipelines: HashMap::new(),
            pbr_shader,
            pbr_texture_layout,
            pbr_pipeline_layout,
            pbr_pipelines: HashMap::new(),
            surface_format: gpu.surface_format(),
            globals_layout,
            globals_buffer,
            globals_bind_group,
            globals_capacity,
            flags_layout,
            flags_buffer,
            flags_bind_group,
            flags_capacity,
            dynamic_buffer,
            dynamic_capacity,
            dynamic_index_buffer,
            dynamic_index_capacity,
            depth: DepthTexture::new(gpu, width, height),
            warned: [false; Warned::COUNT],
            stage_warnings: Stage2dWarnings::default(),
            ghoul2_capture: None,
        }
    }

    /// Arms or disarms the Ghoul2 vertex-stream capture sink. A true `on` starts
    /// a fresh capture, so the next frame records each decoded Ghoul2 surface
    /// stream. The differential vertex golden uses it to bake a fixture.
    pub fn set_ghoul2_capture(&mut self, on: bool) {
        self.ghoul2_capture = if on { Some(Vec::new()) } else { None };
    }

    /// Takes the captured Ghoul2 vertex streams and disarms the sink. The result
    /// is empty when the sink was never armed.
    pub fn take_ghoul2_capture(&mut self) -> Vec<Ghoul2SurfaceCapture> {
        self.ghoul2_capture.take().unwrap_or_default()
    }

    /// Recreates the depth texture on a target resize.
    pub fn resize(&mut self, gpu: &Gpu, width: u32, height: u32) {
        self.depth.resize(gpu, width, height);
    }

    /// Draws the sorted world draw-surf list, one pass per active stage per
    /// surface in stage order (`RB_IterateStagesGeneric`). Each surface resolves
    /// its shader through `R_DecomposeSort`. A sky-shader surface forks into the
    /// sky-box and cloud chain (`RB_StageIteratorSky`), drawn inline at the
    /// surface's sorted position under the far-plane depth range. Non-world
    /// entries are counted and skipped.
    ///
    /// `frame` and `sky` thread the sky-box scratch (`RB_StageIteratorSky` reads
    /// the portal guards off `frame` and reuses `sky`'s cloud tex-coord tables).
    ///
    /// `float_time` is the scene shader clock in seconds (`refdef.floatTime`),
    /// and `noise` drives the waveform generators. A stage with dynamic
    /// texcoords or colours evaluates its own vertices into the per-frame
    /// dynamic buffer, so this fn writes that buffer before the pass reads it.
    ///
    /// `view`, `entities` and `scratch` build the per-entity clip matrices. Each
    /// draw surf's sort key decodes to an entity number. The world entity uses
    /// the view matrix (`view.world.modelMatrix`). A real entity uses
    /// `R_RotateForEntity`'s model matrix against its `trRefEntity_t` row. Every
    /// distinct entity number gets one aligned slot in the globals buffer, and
    /// the draw picks its matrix with a dynamic offset.
    ///
    /// The pass clears the depth buffer to 1.0 per view but loads the color
    /// target, because `Gpu::begin_frame` already cleared color for the frame.
    /// A later 2D pass or a second scene draws over the world.
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &mut self,
        gpu: &Gpu,
        target: &TextureView,
        draw_surfs: &[DrawSurf<SurfaceGeometry>],
        geometry: &WorldGeometry,
        assets: &RenderAssets,
        gpu_images: &mut GpuImages,
        noise: &NoiseState,
        float_time: f32,
        view: &viewParms_t,
        entities: &[trRefEntity_t],
        scratch: &mut TrMainScratch,
        models: &RenderModels,
        g2: &mut Ghoul2System,
        frame: &mut FrameState,
        sky: &mut SkyState,
        fogs: &[fog_t],
        cvars: RenderCvarSnapshot,
    ) -> WorldStats {
        let mut stats = WorldStats::default();

        // The backend this frame draws with, read once (DEC-37 ruling 5). The
        // `Faithful` arm at every seam below is the exact pre-seam path, so a
        // default frame is byte-for-byte unchanged. The `Pbr` arm swaps the
        // pipeline and the texture bind group at those same seams.
        let mode = BackendMode::from_snapshot(cvars);

        // Build one clip matrix per distinct entity number this scene draws,
        // slot 0 the world, then upload them to the dynamic-offset globals
        // buffer. The slot map tags each draw item with its own offset. The
        // per-slot orientations feed the fog tex-coord math.
        let (clips, oris, slot_map) =
            build_entity_slots(draw_surfs, assets, view, entities, scratch);
        self.reserve_globals(gpu, clips.len());
        self.write_globals(gpu, &clips);

        let mut dynamic_vertices: Vec<WorldVertex> = Vec::new();
        let mut dynamic_indices: Vec<u32> = Vec::new();
        let items = self.collect_stage_items(
            draw_surfs,
            geometry,
            assets,
            noise,
            float_time,
            &mut dynamic_vertices,
            &mut dynamic_indices,
            &mut stats,
            &slot_map,
            entities,
            models,
            g2,
            frame,
            sky,
            view,
            fogs,
            &oris,
            cvars,
        );

        if items.is_empty() {
            return stats;
        }

        // Every pipeline the batch uses must exist before the pass borrows
        // `self` immutably. The mode picks which backend's cache is filled.
        for item in &items {
            match mode {
                BackendMode::Faithful => self.ensure_pipeline(gpu, item.key),
                BackendMode::Pbr => self.ensure_pbr_pipeline(gpu, item.key),
            }
        }

        self.reserve_flags(gpu, items.len());
        self.write_flags(gpu, &items, mode);

        // The dynamic buffer holds every dynamic stage's evaluated vertices for
        // this frame, addressed by each item's `base_vertex`.
        if !dynamic_vertices.is_empty() {
            self.reserve_dynamic(gpu, dynamic_vertices.len());
            gpu.queue().write_buffer(
                &self.dynamic_buffer,
                0,
                bytemuck::cast_slice(&dynamic_vertices),
            );
        }

        // The MD3 entity surfaces decoded their indices into this buffer.
        if !dynamic_indices.is_empty() {
            self.reserve_dynamic_indices(gpu, dynamic_indices.len());
            gpu.queue().write_buffer(
                &self.dynamic_index_buffer,
                0,
                bytemuck::cast_slice(&dynamic_indices),
            );
        }

        // One bind group per distinct image pair, reused across the passes that
        // share it. Most surfaces in a frame repeat the same diffuse-plus-lightmap
        // pair, so the cache holds the allocation count near surface count rather
        // than stage-times-surface count.
        let mut group_cache: HashMap<(Option<ImageHandle>, Option<ImageHandle>), usize> =
            HashMap::new();
        let mut bind_groups: Vec<wgpu::BindGroup> = Vec::new();
        let mut item_group: Vec<usize> = Vec::with_capacity(items.len());
        for item in &items {
            let cache_key = (item.diffuse, item.lightmap);
            let group_index = *group_cache.entry(cache_key).or_insert_with(|| {
                // The `Faithful` arm builds the exact two-texture group the
                // pre-seam path built. The `Pbr` arm builds the four-texture
                // group and resolves sidecar-or-neutral (D21).
                let group = match mode {
                    BackendMode::Faithful => gpu_images.world_bind_group(
                        gpu,
                        &self.texture_layout,
                        item.diffuse,
                        item.lightmap,
                    ),
                    BackendMode::Pbr => gpu_images.pbr_world_bind_group(
                        gpu,
                        &self.pbr_texture_layout,
                        item.diffuse,
                        item.lightmap,
                    ),
                };
                bind_groups.push(group);
                bind_groups.len() - 1
            });
            item_group.push(group_index);
        }

        // `RB_BeginDrawingView` decides whether the world pass clears the color
        // target. `Gpu::begin_frame` already cleared it once, and a second scene
        // in the same frame (for example MP cgame's `CG_Draw3DModel`) must draw
        // over the first, so the default is `Load`. Under `r_fastsky` the oracle
        // clears the color instead. The portal-sky arm clears to gray only for
        // the portal-sky refdef. The world arm clears to black when the refdef
        // draws a world and the frame is not the glow pass. The `_DEBUG` tan
        // clear color is dead for the retail build. The oracle sets the scene
        // viewport and scissor before the clear, so its clear covers the scene
        // rect only. This clear covers the whole target, so a sub-viewport scene
        // under `r_fastsky` is the one case that differs.
        // Source: oracle/codemp/renderer/tr_backend.cpp:508-531
        let clear_color: Option<wgpu::Color> = if frame.skyboxportal != 0 {
            let portal_sky = frame.refdef.rdflags & RDF_SKYBOXPORTAL != 0;
            let want_clear = cvars.fastsky != 0 || frame.refdef.rdflags & RDF_NOWORLDMODEL != 0;
            if portal_sky && want_clear {
                Some(wgpu::Color {
                    r: 0.5,
                    g: 0.5,
                    b: 0.5,
                    a: 1.0,
                })
            } else {
                None
            }
        } else if cvars.fastsky != 0
            && frame.refdef.rdflags & RDF_NOWORLDMODEL == 0
            && !frame.render_glowing_objects
        {
            Some(wgpu::Color::BLACK)
        } else {
            None
        };
        let color_load = match clear_color {
            Some(color) => wgpu::LoadOp::Clear(color),
            None => wgpu::LoadOp::Load,
        };

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
                        load: color_load,
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

            // The sky box and clouds draw at the far plane. wgpu forces that
            // through the viewport depth range, not a fixed-function call, so the
            // loop calls `set_viewport(.., 1.0, 1.0)` when it enters the sky run
            // and restores `0.0..1.0` when it leaves. A frame with no sky never
            // sets a viewport, keeping the default full-target range. The rect
            // comes from the view, as the oracle's `qglViewport` does, so a
            // future sub-rect view keeps its inset through the toggle.
            // Source: oracle/codemp/renderer/tr_sky.cpp:808-816,843-844
            // Source: oracle/codemp/renderer/tr_backend.cpp:463-464
            let vp_x = view.viewportX as f32;
            let vp_w = view.viewportWidth as f32;
            let vp_h = view.viewportHeight as f32;
            // `viewParms_t::viewportY` carries the unflipped, 0-at-the-top y that
            // `R_RenderView` stores from `tr.refdef.y`. GL puts 0 at the bottom,
            // so the oracle flips y when it sets the viewport. wgpu puts viewport
            // y at the top like the refdef, so the oracle's GL flip has no
            // counterpart here. We use the stored value straight.
            // Source: oracle/codemp/renderer/tr_scene.cpp:838
            let vp_y = view.viewportY as f32;
            let mut depth_far = false;
            let mut depth_range = DepthRange::Normal;

            for (draw_index, item) in items.iter().enumerate() {
                if item.depth_far != depth_far || item.depth_range != depth_range {
                    depth_far = item.depth_far;
                    depth_range = item.depth_range;
                    // The sky's far-plane window overrides the entity hack: a
                    // sky surface never belongs to a view-model entity.
                    let (min_depth, max_depth) = if depth_far {
                        (1.0, 1.0)
                    } else {
                        depth_range.window()
                    };
                    pass.set_viewport(vp_x, vp_y, vp_w, vp_h, min_depth, max_depth);
                }

                let pipeline = match mode {
                    BackendMode::Faithful => &self.pipelines,
                    BackendMode::Pbr => &self.pbr_pipelines,
                }
                .get(&item.key)
                .expect("world pipeline was created for every item's key above");
                let offset = (draw_index as u64 * SURFACE_FLAGS_STRIDE) as u32;

                // A dynamic stage draws from the per-frame buffer, a static
                // stage from the concatenated world buffer.
                let vertex_buffer = if item.dynamic {
                    &self.dynamic_buffer
                } else {
                    &geometry.vertex_buffer
                };

                // An MD3 entity surface indexes the per-frame index buffer, a
                // world surface the static world index buffer.
                let index_buffer = if item.index_dynamic {
                    &self.dynamic_index_buffer
                } else {
                    &geometry.index_buffer
                };
                pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);

                pass.set_pipeline(pipeline);
                // Group 0 selects this surface's clip matrix by its entity slot.
                pass.set_bind_group(0, &self.globals_bind_group, &[item.globals_offset]);
                pass.set_bind_group(1, &bind_groups[item_group[draw_index]], &[]);
                pass.set_bind_group(2, &self.flags_bind_group, &[offset]);
                pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                pass.draw_indexed(
                    item.first_index..item.first_index + item.index_count,
                    item.base_vertex,
                    0..1,
                );

                stats.draw_calls += 1;
                if item.reads_lightmap {
                    stats.lightmapped += 1;
                }
            }
        }
        gpu.queue().submit(std::iter::once(encoder.finish()));

        stats
    }

    /// Resolves every world draw surf into its per-stage [`StageDrawItem`]
    /// passes, in stage order, and builds any dynamic vertex blocks. Non-world
    /// and empty entries are counted into `stats`, and a sky-parms entry forks
    /// into the sky-box and cloud chain (`collect_sky_surface`).
    ///
    //TODO: Port ProjectDlightTexture
    // Source: oracle/codemp/renderer/tr_shade.cpp:840-1010
    // `RB_StageIteratorGeneric` appends one additive pass per dynamic light
    // that the surface's `dlightBits` marks. The scene's dlights reach
    // `tr.refdef.dlights` and the sort keys carry the dlight map, so the
    // frontend half is live; the pass itself picks a texcoord axis from
    // `tess.normal[i]`, and `WorldVertex` carries no normal, so a world surface
    // has nothing to project along. Landing the pass means widening the vertex
    // format first.
    #[allow(clippy::too_many_arguments)]
    fn collect_stage_items(
        &mut self,
        draw_surfs: &[DrawSurf<SurfaceGeometry>],
        geometry: &WorldGeometry,
        assets: &RenderAssets,
        noise: &NoiseState,
        float_time: f32,
        dynamic_vertices: &mut Vec<WorldVertex>,
        dynamic_indices: &mut Vec<u32>,
        stats: &mut WorldStats,
        slot_map: &HashMap<i32, u32>,
        entities: &[trRefEntity_t],
        models: &RenderModels,
        g2: &mut Ghoul2System,
        frame: &mut FrameState,
        sky: &mut SkyState,
        view: &viewParms_t,
        fogs: &[fog_t],
        oris: &[orientationr_t],
        cvars: RenderCvarSnapshot,
    ) -> Vec<StageDrawItem> {
        let mut items: Vec<StageDrawItem> = Vec::new();
        // The two scene-wide inputs the renderfx colour overrides read: the
        // disintegrate threshold clock and the volumetric fade axis.
        // Source: oracle/codemp/renderer/tr_shade.cpp:1568,1596
        let refdef_time = frame.refdef.time;
        let view_axis = view.ori.axis[0];

        for surf in draw_surfs {
            match surf.surface {
                SurfaceGeometry::World(world_ref) => {
                    self.collect_world_surface(
                        surf.sort,
                        world_ref,
                        geometry,
                        assets,
                        noise,
                        float_time,
                        dynamic_vertices,
                        dynamic_indices,
                        stats,
                        slot_map,
                        entities,
                        frame,
                        sky,
                        view,
                        fogs,
                        oris,
                        cvars,
                        refdef_time,
                        view_axis,
                        &mut items,
                    );
                }
                SurfaceGeometry::Md3(md3_ref) => {
                    self.collect_md3_surface(
                        surf.sort,
                        md3_ref,
                        assets,
                        noise,
                        float_time,
                        dynamic_vertices,
                        dynamic_indices,
                        stats,
                        slot_map,
                        entities,
                        models,
                        frame,
                        sky,
                        view,
                        fogs,
                        oris,
                        cvars,
                        refdef_time,
                        view_axis,
                        &mut items,
                    );
                }
                SurfaceGeometry::Ghoul2(g2_ref) => {
                    self.collect_ghoul2_surface(
                        surf.sort,
                        g2_ref,
                        assets,
                        noise,
                        float_time,
                        dynamic_vertices,
                        dynamic_indices,
                        stats,
                        slot_map,
                        entities,
                        models,
                        g2,
                        frame,
                        sky,
                        view,
                        fogs,
                        oris,
                        cvars,
                        refdef_time,
                        view_axis,
                        &mut items,
                    );
                }
                SurfaceGeometry::Poly { verts } => {
                    self.collect_poly_surface(
                        surf.sort,
                        verts,
                        assets,
                        noise,
                        float_time,
                        dynamic_vertices,
                        dynamic_indices,
                        stats,
                        slot_map,
                        entities,
                        view,
                        fogs,
                        oris,
                        refdef_time,
                        view_axis,
                        &mut items,
                    );
                }
                SurfaceGeometry::Other => {
                    self.collect_entity_surface(
                        surf.sort,
                        assets,
                        noise,
                        float_time,
                        dynamic_vertices,
                        dynamic_indices,
                        stats,
                        slot_map,
                        entities,
                        view,
                        fogs,
                        oris,
                        refdef_time,
                        view_axis,
                        &mut items,
                    );
                }
                _ => {
                    stats.skipped_non_world += 1;
                }
            }
        }

        items
    }

    /// Resolves one `SF_POLY` draw surf: the scene polygon `RE_AddPolyToScene`
    /// recorded, fan-triangulated into the per-frame buffers and drawn one pass
    /// per active stage.
    ///
    /// This is the census's mark path. All 443,515 poly submissions across the
    /// four traces carry world-space vertices under the world entity, so the
    /// slot lookup lands on the world clip matrix.
    ///
    /// Source: `oracle/codemp/renderer/tr_surface.cpp:220-254`
    /// (`RB_SurfacePolychain`)
    #[allow(clippy::too_many_arguments)]
    fn collect_poly_surface(
        &mut self,
        sort: u32,
        poly_verts: &[polyVert_t],
        assets: &RenderAssets,
        noise: &NoiseState,
        float_time: f32,
        dynamic_vertices: &mut Vec<WorldVertex>,
        dynamic_indices: &mut Vec<u32>,
        stats: &mut WorldStats,
        slot_map: &HashMap<i32, u32>,
        entities: &[trRefEntity_t],
        view: &viewParms_t,
        fogs: &[fog_t],
        oris: &[orientationr_t],
        refdef_time: i32,
        view_axis: [f32; 3],
        items: &mut Vec<StageDrawItem>,
    ) {
        let (entity_num, shader_handle, fog_num, _dlight_map) =
            R_DecomposeSort(sort, &assets.sorted_shaders);
        let Some(shader) = assets.shaders.get(shader_handle) else {
            return;
        };
        if poly_verts.len() < 3 {
            stats.empty_surfaces += 1;
            return;
        }

        // The fan: vertex 0 anchors every triangle.
        let verts: Vec<WorldVertex> = poly_verts
            .iter()
            .map(|v| WorldVertex {
                position: v.xyz,
                st: v.st,
                lightmap_st: [0.0, 0.0],
                color: v.modulate,
            })
            .collect();
        let mut index_block: Vec<u32> = Vec::with_capacity((poly_verts.len() - 2) * 3);
        for i in 0..poly_verts.len() as u32 - 2 {
            index_block.extend_from_slice(&[0, i + 1, i + 2]);
        }

        let slot = slot_map.get(&entity_num).copied().unwrap_or(0);
        let globals_offset = (slot as u64 * GLOBALS_STRIDE) as u32;
        let entity_float_time = float_time - entity_shader_time(entities, entity_num);
        let surface_fog = resolve_surface_fog(fog_num, fogs, oris, slot, view);
        let entity = entities.get(entity_num as usize);

        let first_index = dynamic_indices.len() as u32;
        let index_count = index_block.len() as u32;
        dynamic_indices.extend_from_slice(&index_block);

        let before = items.len();
        for stage in shader.stages.iter().filter(|stage| stage.active) {
            let item = self.build_cpu_surface_stage_item(
                shader,
                stage,
                &verts,
                first_index,
                index_count,
                entity_float_time,
                noise,
                assets,
                surface_fog,
                entity,
                None,
                dynamic_vertices,
                globals_offset,
                refdef_time,
                view_axis,
            );
            if let Some(item) = item {
                items.push(item);
            }
        }
        let stages_end = items.len();

        if let Some(sf) = surface_fog {
            if shader.fog_pass != FogPass::None {
                let item = self.build_fog_stage_item(
                    &verts,
                    sf,
                    shader.fog_pass,
                    assets.fog_image,
                    first_index,
                    index_count,
                    true,
                    globals_offset,
                    dynamic_vertices,
                    DepthRange::resolve(entity),
                );
                if let Some(item) = item {
                    items.push(item);
                    stats.fog_passes_drawn += 1;
                }
            }
        }

        if stages_end > before {
            stats.surfaces_drawn += 1;
            stats.poly_surfaces_drawn += 1;
        }
    }

    /// Resolves one `SF_ENTITY` draw surf: the generated-geometry family
    /// `RB_SurfaceEntity` dispatches on `reType`. The vertices are built on the
    /// CPU in world space, since `R_RotateForEntity` hands every non-`RT_MODEL`
    /// entity the world orientation, then drawn one dynamic-buffer pass per
    /// active stage.
    ///
    /// A `reType` outside the census set builds no geometry and counts as a
    /// skip, so the remaining generated kinds stay visible in the stats rather
    /// than silently drawing nothing.
    ///
    /// Source: `oracle/codemp/renderer/tr_surface.cpp:1798-1870`
    /// (`RB_SurfaceEntity`), `oracle/codemp/renderer/tr_main.cpp:302-311`
    /// (`R_RotateForEntity`'s non-`RT_MODEL` arm)
    #[allow(clippy::too_many_arguments)]
    fn collect_entity_surface(
        &mut self,
        sort: u32,
        assets: &RenderAssets,
        noise: &NoiseState,
        float_time: f32,
        dynamic_vertices: &mut Vec<WorldVertex>,
        dynamic_indices: &mut Vec<u32>,
        stats: &mut WorldStats,
        slot_map: &HashMap<i32, u32>,
        entities: &[trRefEntity_t],
        view: &viewParms_t,
        fogs: &[fog_t],
        oris: &[orientationr_t],
        refdef_time: i32,
        view_axis: [f32; 3],
        items: &mut Vec<StageDrawItem>,
    ) {
        let (entity_num, shader_handle, fog_num, _dlight_map) =
            R_DecomposeSort(sort, &assets.sorted_shaders);
        let Some(shader) = assets.shaders.get(shader_handle) else {
            return;
        };
        let Some(entity) = entities.get(entity_num as usize) else {
            stats.skipped_non_world += 1;
            return;
        };

        let (verts, index_block) = build_entity_geometry(&entity.e, view);
        if index_block.is_empty() {
            self.warn_once(Warned::EntitySurface);
            stats.skipped_non_world += 1;
            return;
        }

        let slot = slot_map.get(&entity_num).copied().unwrap_or(0);
        let globals_offset = (slot as u64 * GLOBALS_STRIDE) as u32;
        let entity_float_time = float_time - entity_shader_time(entities, entity_num);
        let surface_fog = resolve_surface_fog(fog_num, fogs, oris, slot, view);

        let first_index = dynamic_indices.len() as u32;
        let index_count = index_block.len() as u32;
        dynamic_indices.extend_from_slice(&index_block);

        let before = items.len();
        for stage in shader.stages.iter().filter(|stage| stage.active) {
            let item = self.build_cpu_surface_stage_item(
                shader,
                stage,
                &verts,
                first_index,
                index_count,
                entity_float_time,
                noise,
                assets,
                surface_fog,
                Some(entity),
                None,
                dynamic_vertices,
                globals_offset,
                refdef_time,
                view_axis,
            );
            if let Some(item) = item {
                items.push(item);
            }
        }
        let stages_end = items.len();

        if let Some(sf) = surface_fog {
            if shader.fog_pass != FogPass::None {
                let item = self.build_fog_stage_item(
                    &verts,
                    sf,
                    shader.fog_pass,
                    assets.fog_image,
                    first_index,
                    index_count,
                    true,
                    globals_offset,
                    dynamic_vertices,
                    DepthRange::resolve(Some(entity)),
                );
                if let Some(item) = item {
                    items.push(item);
                    stats.fog_passes_drawn += 1;
                }
            }
        }

        if stages_end > before {
            stats.surfaces_drawn += 1;
            stats.entity_surfaces_drawn += 1;
        }
    }

    /// Resolves one world (BSP) draw surf into its per-stage passes, appending
    /// them to `items`. The per-item shader clock is `floatTime - e.shaderTime`
    /// for the surface's entity, so an inline brush model with a `shaderTime`
    /// animates from its own offset.
    #[allow(clippy::too_many_arguments)]
    fn collect_world_surface(
        &mut self,
        sort: u32,
        world_ref: WorldSurfaceRef,
        geometry: &WorldGeometry,
        assets: &RenderAssets,
        noise: &NoiseState,
        float_time: f32,
        dynamic_vertices: &mut Vec<WorldVertex>,
        dynamic_indices: &mut Vec<u32>,
        stats: &mut WorldStats,
        slot_map: &HashMap<i32, u32>,
        entities: &[trRefEntity_t],
        frame: &mut FrameState,
        sky: &mut SkyState,
        view: &viewParms_t,
        fogs: &[fog_t],
        oris: &[orientationr_t],
        cvars: RenderCvarSnapshot,
        refdef_time: i32,
        view_axis: [f32; 3],
        items: &mut Vec<StageDrawItem>,
    ) {
        let index = world_ref_index(world_ref);
        let range = geometry.range(index);
        if range.index_count == 0 {
            stats.empty_surfaces += 1;
            return;
        }

        let (entity_num, shader_handle, fog_num, _dlight_map) =
            R_DecomposeSort(sort, &assets.sorted_shaders);
        let Some(shader) = assets.shaders.get(shader_handle) else {
            return;
        };

        // Slot 0 is the world. A draw surf with an unmapped entity number uses
        // slot 0. `build_entity_slots` maps every entity number the draw list
        // carries, so an unmapped number does not occur here.
        let slot = slot_map.get(&entity_num).copied().unwrap_or(0);
        let globals_offset = (slot as u64 * GLOBALS_STRIDE) as u32;
        let entity_float_time = float_time - entity_shader_time(entities, entity_num);
        let surface_fog = resolve_surface_fog(fog_num, fogs, oris, slot, view);

        let cpu_start = range.base_vertex as usize;
        let cpu = &geometry.cpu_vertices[cpu_start..cpu_start + range.vertex_count as usize];

        // A sky shader forks into the sky-box and cloud chain, drawn inline at
        // this surface's sorted position (RB_BeginSurface's fork on
        // shader->sky). The oracle batches contiguous same-shader surfaces into
        // one tess and runs RB_StageIteratorSky once for that run (RB_EndSurface).
        // This backend runs the iterator once per surface instead.
        //
        // The outer-box faces draw with GL_State(0) and depth writes off, so a
        // repeat per surface is idempotent and the drawn union covers the same
        // sky-box area. The cloud layer is not idempotent. It carries the sky
        // shader stage's own blend, so a blended or additive cloud stage
        // composites once per sky surface and compounds over the surfaces. This
        // is a preserved behavioral quirk of the per-surface shape, not the
        // oracle's per-run batch.
        // Source: oracle/codemp/renderer/tr_shade.cpp:362-372 (RB_BeginSurface),
        // 2391-2432 (RB_EndSurface runs the sky iterator once per batch)
        if let Some(sky_parms) = &shader.sky {
            let verts: Vec<vec3_t> = cpu.iter().map(|v| v.position).collect();
            let idx_start = range.first_index as usize;
            let indexes =
                &geometry.cpu_indices[idx_start..idx_start + range.index_count as usize];
            self.collect_sky_surface(
                shader,
                sky_parms,
                &verts,
                indexes,
                entity_float_time,
                noise,
                assets,
                frame,
                sky,
                view,
                globals_offset,
                dynamic_vertices,
                dynamic_indices,
                stats,
                cvars,
                refdef_time,
                view_axis,
                items,
            );
            return;
        }

        // A bezier-patch grid subsamples its lattice per frame by the
        // view-relative LOD error (RB_SurfaceGrid). The grid verts already sit
        // in the shared world buffer, so the subsample only picks which of them
        // the indices reference. A patch whose LOD keeps every row and column
        // produces the identical index set the static full-res path uploaded at
        // level load, so that case keeps the static range and emits nothing.
        // The reduced indices are relative offsets into the surface's full
        // vertex block, so they address the static block, a per-stage dynamic
        // block, or the fog block alike (every vertex path copies the block in
        // full order).
        // Source: oracle/codemp/renderer/tr_surface.cpp:1594-1638,1740-1755
        let mut draw_range = range;
        let mut index_dynamic = false;
        if let (Some(grid), Some(ori)) =
            (world_surface_grid(assets, index), oris.get(slot as usize))
        {
            let lod_indices = grid_lod_indices(grid, ori, view, cvars.lod_curve_error);
            // A reduced lattice has strictly fewer indices. An equal count is
            // the keep-everything case, which the static path draws identically.
            if (lod_indices.len() as u32) < range.index_count {
                draw_range.first_index = dynamic_indices.len() as u32;
                draw_range.index_count = lod_indices.len() as u32;
                dynamic_indices.extend_from_slice(&lod_indices);
                index_dynamic = true;
            }
        }

        let before = items.len();
        for stage in shader.stages.iter().filter(|stage| stage.active) {
            let item = self.build_stage_item(
                shader,
                stage,
                &draw_range,
                index_dynamic,
                cpu,
                assets,
                noise,
                entity_float_time,
                surface_fog,
                entities.get(entity_num as usize),
                dynamic_vertices,
                globals_offset,
                refdef_time,
                view_axis,
            );
            // A surface-sprite stage draws nothing here, so it yields no item.
            if let Some(item) = item {
                items.push(item);
            }
        }

        // The fog pass draws at the tail of the stage list when the surface is
        // fogged and the shader declares a fogPass, over the static world index
        // block. `RB_StageIteratorGeneric` runs it once after every stage.
        // DEFERRED: R4 - the oracle gate drops two sub-conditions here:
        // `tess.fogNum != tr.world->globalFog` and `r_drawfog->value != 2`.
        // r_drawfog ships at 2, so at the default the oracle fogs a global-fog
        // surface with hardware GL fog, not this image pass. This pass stands in
        // for GL fog until GLFog lands, which is the `r_drawfog 1` look.
        // Source: oracle/codemp/renderer/tr_shade.cpp:2344, 1960-1961
        // The tally counts stage draws only, so a fog-only tail cannot mark
        // the surface as drawn.
        let stages_end = items.len();

        if let Some(sf) = surface_fog {
            if shader.fog_pass != FogPass::None {
                let item = self.build_fog_stage_item(
                    cpu,
                    sf,
                    shader.fog_pass,
                    assets.fog_image,
                    draw_range.first_index,
                    draw_range.index_count,
                    index_dynamic,
                    globals_offset,
                    dynamic_vertices,
                    DepthRange::resolve(entities.get(entity_num as usize)),
                );
                if let Some(item) = item {
                    items.push(item);
                    stats.fog_passes_drawn += 1;
                }
            }
        }

        if stages_end > before {
            stats.surfaces_drawn += 1;
            if entity_num != TR_WORLDENT {
                stats.entity_surfaces_drawn += 1;
            }
        }
    }

    /// Resolves one MD3 (`MOD_MESH`) entity draw surf: it decodes the surface's
    /// vertices on the CPU per frame (`LerpMeshVertexes`), packs its indices
    /// into the per-frame index buffer, then draws one dynamic-buffer pass per
    /// active stage with the entity's per-entity clip matrix and shader clock.
    ///
    /// Source: `oracle/codemp/renderer/tr_surface.cpp:1235-1397`
    /// (`LerpMeshVertexes`/`RB_SurfaceMesh`)
    #[allow(clippy::too_many_arguments)]
    fn collect_md3_surface(
        &mut self,
        sort: u32,
        md3_ref: Md3SurfaceRef,
        assets: &RenderAssets,
        noise: &NoiseState,
        float_time: f32,
        dynamic_vertices: &mut Vec<WorldVertex>,
        dynamic_indices: &mut Vec<u32>,
        stats: &mut WorldStats,
        slot_map: &HashMap<i32, u32>,
        entities: &[trRefEntity_t],
        models: &RenderModels,
        frame: &mut FrameState,
        sky: &mut SkyState,
        view: &viewParms_t,
        fogs: &[fog_t],
        oris: &[orientationr_t],
        cvars: RenderCvarSnapshot,
        refdef_time: i32,
        view_axis: [f32; 3],
        items: &mut Vec<StageDrawItem>,
    ) {
        let (entity_num, shader_handle, fog_num, _dlight_map) =
            R_DecomposeSort(sort, &assets.sorted_shaders);
        let Some(shader) = assets.shaders.get(shader_handle) else {
            return;
        };

        let slot = slot_map.get(&entity_num).copied().unwrap_or(0);
        let globals_offset = (slot as u64 * GLOBALS_STRIDE) as u32;
        let entity_float_time = float_time - entity_shader_time(entities, entity_num);
        let surface_fog = resolve_surface_fog(fog_num, fogs, oris, slot, view);
        // The identity-light default vertex colour is the entity's shaderRGBA.
        let rgba = entities
            .get(entity_num as usize)
            .map(|ent| ent.e.shaderRGBA)
            .unwrap_or([255, 255, 255, 255]);

        // Decode the keyframe-lerped vertices, normals, and triangle indices.
        let Some((md3_vertices, md3_index_block, md3_normals)) =
            decode_md3_surface(models, md3_ref, rgba, &assets.function_tables.sin_table)
        else {
            stats.md3_decode_failed += 1;
            return;
        };
        let entity = entities.get(entity_num as usize);
        if md3_index_block.is_empty() {
            stats.empty_surfaces += 1;
            return;
        }

        // A sky shader forks into the sky chain (RB_BeginSurface's fork on
        // shader->sky), the same as the world path. An MD3 surface hands its
        // decoded vertices in model space, since the entity transform lives in
        // the per-entity clip matrix, not the CPU positions. No shipped content
        // puts a sky shader on an entity model, so this arm stays untested by a
        // live scene.
        if let Some(sky_parms) = &shader.sky {
            let verts: Vec<vec3_t> = md3_vertices.iter().map(|v| v.position).collect();
            self.collect_sky_surface(
                shader,
                sky_parms,
                &verts,
                &md3_index_block,
                entity_float_time,
                noise,
                assets,
                frame,
                sky,
                view,
                globals_offset,
                dynamic_vertices,
                dynamic_indices,
                stats,
                cvars,
                refdef_time,
                view_axis,
                items,
            );
            return;
        }

        // One shared index block per surface, in the per-frame index buffer.
        let first_index = dynamic_indices.len() as u32;
        let index_count = md3_index_block.len() as u32;
        dynamic_indices.extend_from_slice(&md3_index_block);

        let before = items.len();
        for stage in shader.stages.iter().filter(|stage| stage.active) {
            let item = self.build_cpu_surface_stage_item(
                shader,
                stage,
                &md3_vertices,
                first_index,
                index_count,
                entity_float_time,
                noise,
                assets,
                surface_fog,
                entity,
                Some(&md3_normals),
                dynamic_vertices,
                globals_offset,
                refdef_time,
                view_axis,
            );
            if let Some(item) = item {
                items.push(item);
            }
        }

        // The fog pass draws at the tail, over the same per-frame index block.
        // DEFERRED: R4 - the oracle gate drops two sub-conditions here:
        // `tess.fogNum != tr.world->globalFog` and `r_drawfog->value != 2`.
        // r_drawfog ships at 2, so at the default the oracle fogs a global-fog
        // surface with hardware GL fog, not this image pass. This pass stands in
        // for GL fog until GLFog lands, which is the `r_drawfog 1` look.
        // Source: oracle/codemp/renderer/tr_shade.cpp:2344, 1960-1961
        // The tally counts stage draws only, so a fog-only tail cannot mark
        // the surface as drawn.
        let stages_end = items.len();

        if let Some(sf) = surface_fog {
            if shader.fog_pass != FogPass::None {
                let item = self.build_fog_stage_item(
                    &md3_vertices,
                    sf,
                    shader.fog_pass,
                    assets.fog_image,
                    first_index,
                    index_count,
                    true,
                    globals_offset,
                    dynamic_vertices,
                    DepthRange::resolve(entity),
                );
                if let Some(item) = item {
                    items.push(item);
                    stats.fog_passes_drawn += 1;
                }
            }
        }

        if stages_end > before {
            stats.surfaces_drawn += 1;
            stats.md3_surfaces_drawn += 1;
            if entity_num != TR_WORLDENT {
                stats.entity_surfaces_drawn += 1;
            }
        }
    }

    /// Resolves one Ghoul2 (`MOD_MDXM`) entity draw surf: it deforms the
    /// surface's vertices on the CPU per frame by the lerped bone matrices
    /// (`RB_SurfaceGhoul`, the non-gore main arm), packs its indices into the
    /// per-frame index buffer, then draws one dynamic-buffer pass per active
    /// stage with the entity's per-entity clip matrix and shader clock.
    ///
    /// The shared per-stage build is [`Self::build_cpu_surface_stage_item`], the
    /// same path the MD3 surface uses.
    ///
    /// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:4060-4451` (the non-gore
    /// main arm)
    #[allow(clippy::too_many_arguments)]
    fn collect_ghoul2_surface(
        &mut self,
        sort: u32,
        g2_ref: G2SurfaceRef,
        assets: &RenderAssets,
        noise: &NoiseState,
        float_time: f32,
        dynamic_vertices: &mut Vec<WorldVertex>,
        dynamic_indices: &mut Vec<u32>,
        stats: &mut WorldStats,
        slot_map: &HashMap<i32, u32>,
        entities: &[trRefEntity_t],
        models: &RenderModels,
        g2: &mut Ghoul2System,
        frame: &mut FrameState,
        sky: &mut SkyState,
        view: &viewParms_t,
        fogs: &[fog_t],
        oris: &[orientationr_t],
        cvars: RenderCvarSnapshot,
        refdef_time: i32,
        view_axis: [f32; 3],
        items: &mut Vec<StageDrawItem>,
    ) {
        let (entity_num, shader_handle, fog_num, _dlight_map) =
            R_DecomposeSort(sort, &assets.sorted_shaders);
        let Some(shader) = assets.shaders.get(shader_handle) else {
            return;
        };

        let slot = slot_map.get(&entity_num).copied().unwrap_or(0);
        let globals_offset = (slot as u64 * GLOBALS_STRIDE) as u32;
        let entity_float_time = float_time - entity_shader_time(entities, entity_num);
        let surface_fog = resolve_surface_fog(fog_num, fogs, oris, slot, view);
        // The identity-light default vertex colour is the entity's shaderRGBA.
        let rgba = entities
            .get(entity_num as usize)
            .map(|ent| ent.e.shaderRGBA)
            .unwrap_or([255, 255, 255, 255]);

        // Deform the surface by the lerped bones, decode the bone-0 normals, and
        // read the triangle indices.
        let Some((g2_vertices, g2_index_block, g2_normals)) =
            decode_ghoul2_surface(models, g2, g2_ref, rgba)
        else {
            stats.ghoul2_decode_failed += 1;
            return;
        };

        // Test-only capture: record the decoded stream in draw-surf order for the
        // differential vertex golden. The sink stays `None` in normal use.
        if let Some(capture) = self.ghoul2_capture.as_mut() {
            capture.push((
                g2_vertices.clone(),
                g2_index_block.clone(),
                g2_normals.clone(),
            ));
        }

        if g2_index_block.is_empty() {
            stats.empty_surfaces += 1;
            return;
        }

        // A sky shader forks into the sky chain (RB_BeginSurface's fork on
        // shader->sky), the same as the world path. A Ghoul2 surface hands its
        // bone-deformed vertices in model space, since the entity transform
        // lives in the per-entity clip matrix. No shipped content puts a sky
        // shader on an entity model, so this arm stays untested by a live scene.
        if let Some(sky_parms) = &shader.sky {
            let verts: Vec<vec3_t> = g2_vertices.iter().map(|v| v.position).collect();
            self.collect_sky_surface(
                shader,
                sky_parms,
                &verts,
                &g2_index_block,
                entity_float_time,
                noise,
                assets,
                frame,
                sky,
                view,
                globals_offset,
                dynamic_vertices,
                dynamic_indices,
                stats,
                cvars,
                refdef_time,
                view_axis,
                items,
            );
            return;
        }

        // One shared index block per surface, in the per-frame index buffer.
        let first_index = dynamic_indices.len() as u32;
        let index_count = g2_index_block.len() as u32;
        dynamic_indices.extend_from_slice(&g2_index_block);

        let entity = entities.get(entity_num as usize);
        let before = items.len();
        for stage in shader.stages.iter().filter(|stage| stage.active) {
            let item = self.build_cpu_surface_stage_item(
                shader,
                stage,
                &g2_vertices,
                first_index,
                index_count,
                entity_float_time,
                noise,
                assets,
                surface_fog,
                entity,
                Some(&g2_normals),
                dynamic_vertices,
                globals_offset,
                refdef_time,
                view_axis,
            );
            if let Some(item) = item {
                items.push(item);
            }
        }

        // The fog pass draws at the tail, over the same per-frame index block.
        // DEFERRED: R4 - the oracle gate drops two sub-conditions here:
        // `tess.fogNum != tr.world->globalFog` and `r_drawfog->value != 2`.
        // r_drawfog ships at 2, so at the default the oracle fogs a global-fog
        // surface with hardware GL fog, not this image pass. This pass stands in
        // for GL fog until GLFog lands, which is the `r_drawfog 1` look.
        // Source: oracle/codemp/renderer/tr_shade.cpp:2344, 1960-1961
        // The tally counts stage draws only, so a fog-only tail cannot mark
        // the surface as drawn.
        let stages_end = items.len();

        if let Some(sf) = surface_fog {
            if shader.fog_pass != FogPass::None {
                let item = self.build_fog_stage_item(
                    &g2_vertices,
                    sf,
                    shader.fog_pass,
                    assets.fog_image,
                    first_index,
                    index_count,
                    true,
                    globals_offset,
                    dynamic_vertices,
                    DepthRange::resolve(entity),
                );
                if let Some(item) = item {
                    items.push(item);
                    stats.fog_passes_drawn += 1;
                }
            }
        }

        if stages_end > before {
            stats.surfaces_drawn += 1;
            stats.ghoul2_surfaces_drawn += 1;
            if entity_num != TR_WORLDENT {
                stats.entity_surfaces_drawn += 1;
            }
        }
    }

    /// Draws one sky-shader surface's box and clouds inline (`RB_StageIteratorSky`).
    /// The surface's triangles project onto the sky box. Each visible outer-box
    /// face binds its image and draws its grid, and the cloud layer feeds the
    /// generic stage machinery, one pass per active stage. Every sky pass carries
    /// `depth_far`, so the draw loop forces the far-plane depth range around them.
    ///
    /// The sky box draws with `GL_State(0)`: no blend, depth compare
    /// less-or-equal, depth writes off. The vertex colour is
    /// `qglColor3f(tr.identityLight, ...)`.
    ///
    /// Source: `oracle/codemp/renderer/tr_sky.cpp:786-848`
    #[allow(clippy::too_many_arguments)]
    fn collect_sky_surface(
        &mut self,
        shader: &ShaderAsset,
        sky_parms: &SkyParms,
        verts: &[vec3_t],
        indexes: &[u32],
        float_time: f32,
        noise: &NoiseState,
        assets: &RenderAssets,
        frame: &mut FrameState,
        sky: &mut SkyState,
        view: &viewParms_t,
        globals_offset: u32,
        dynamic_vertices: &mut Vec<WorldVertex>,
        dynamic_indices: &mut Vec<u32>,
        stats: &mut WorldStats,
        cvars: RenderCvarSnapshot,
        refdef_time: i32,
        view_axis: [f32; 3],
        items: &mut Vec<StageDrawItem>,
    ) {
        // Project the surface onto the sky box and build the cloud geometry. A
        // tripped guard (glow pass, sky-box portal) draws no sky.
        //
        // `r_fastsky` returns before `RB_StageIteratorSky` runs, so the sky
        // never draws and `skyRenderedThisView` stays unwritten. The oracle
        // gate sits between the glow-pass check and the sky-box-portal check
        // inside `RB_StageIteratorSky`. This guard runs before the call, which
        // keeps the same order against the `skyRenderedThisView` write.
        // Source: oracle/codemp/renderer/tr_sky.cpp:791-793
        if cvars.fastsky != 0 {
            return;
        }

        let Some(data) = RB_StageIteratorSky(
            frame,
            sky,
            sky_parms,
            shader.num_unfogged_passes,
            assets.default_image,
            verts,
            indexes,
            view,
        ) else {
            return;
        };

        // RB_DrawSun draws nothing here, to match the oracle. The oracle's only
        // RB_DrawSun call site sits inside an `#if 0` block
        // (tr_backend.cpp:1222-1224), so retail Jedi Academy never draws the sun.
        // Parity is no live draw.
        // Source: oracle/codemp/renderer/tr_sky.cpp:846-847

        let before = items.len();

        // The GL_State(0) sky-box pass: no blend, depth compare less-or-equal,
        // depth writes off.
        let box_key = PipelineKey {
            blend: blend_state_from_gls(0),
            depth_equal: false,
            depth_write: false,
        };
        let color = sky_identity_color();

        // Each visible outer-box face binds its image and draws its grid strips.
        for face in data.faces.iter().flatten() {
            let (first_index, index_count, base_vertex) = build_sky_face_block(
                face,
                view.ori.origin,
                color,
                dynamic_vertices,
                dynamic_indices,
            );
            if index_count == 0 {
                continue;
            }
            items.push(StageDrawItem {
                key: box_key,
                diffuse: face.image,
                lightmap: None,
                mode: MODE_SINGLE,
                tex_from_lightmap: false,
                alpha_func: 0,
                reads_lightmap: false,
                first_index,
                index_count,
                base_vertex,
                dynamic: true,
                index_dynamic: true,
                globals_offset,
                depth_far: true,
                depth_range: DepthRange::Normal,
            });
        }

        // The cloud layer feeds the generic stage machinery, one pass per active
        // stage of the sky shader (the oracle hands its cloud tess to
        // RB_StageIteratorGeneric).
        if !data.cloud.indexes.is_empty() {
            let cloud_first = dynamic_indices.len() as u32;
            dynamic_indices.extend_from_slice(&data.cloud.indexes);
            let cloud_count = data.cloud.indexes.len() as u32;

            // The cloud xyz already bakes in the view origin (FillCloudySkySide).
            // The stage evaluators overwrite the colour, so the input is the
            // identity white the tess starts at.
            let cloud_cpu: Vec<WorldVertex> = data
                .cloud
                .xyz
                .iter()
                .zip(&data.cloud.tex_coords)
                .map(|(xyz, st)| WorldVertex {
                    position: *xyz,
                    st: *st,
                    lightmap_st: *st,
                    color: [255, 255, 255, 255],
                })
                .collect();

            for stage in shader.stages.iter().filter(|stage| stage.active) {
                let item = self.build_cpu_surface_stage_item(
                    shader,
                    stage,
                    &cloud_cpu,
                    cloud_first,
                    cloud_count,
                    float_time,
                    noise,
                    assets,
                    // The sky box and clouds draw at the far plane, never fogged.
                    None,
                    // The sky path carries no entity and no per-vertex normal.
                    None,
                    None,
                    dynamic_vertices,
                    globals_offset,
                    refdef_time,
                    view_axis,
                );
                if let Some(mut item) = item {
                    item.depth_far = true;
                    items.push(item);
                }
            }
        }

        if items.len() > before {
            stats.surfaces_drawn += 1;
            stats.sky_surfaces_drawn += 1;
        }
    }

    /// Resolves one active stage into its draw pass, or `None` when the stage
    /// draws nothing here. The two-texture path is the `GL_MODULATE` collapse
    /// (`DrawMultitextured`), the single-texture path is the common
    /// `R_DrawElements` arm. A stage with tcMods, a waveform gen, or a colour
    /// gen other than a vertex pass-through evaluates its vertices into
    /// `dynamic_vertices` and draws from there.
    ///
    /// A surface-sprite stage yields `None`. `RB_IterateStagesGeneric` skips it
    /// with a `continue` because `RB_DrawSurfaceSprites` handles sprites after
    /// every other stage.
    ///
    /// `index_dynamic` selects the per-frame index buffer over the static world
    /// index buffer. A LOD-subsampled grid sets it: its reduced indices are
    /// relative offsets into the surface's full vertex block, so a static-vertex
    /// stage still points `base_vertex` at that block and the reduced indices
    /// address it.
    ///
    /// Source: `oracle/codemp/renderer/tr_shade.cpp:2136-2158` (multitexture vs
    /// single), `oracle/codemp/renderer/tr_shade.cpp:394-441` (`DrawMultitextured`),
    /// `oracle/codemp/renderer/tr_shade.cpp:2055-2059` (surface-sprite skip)
    #[allow(clippy::too_many_arguments)]
    fn build_stage_item(
        &mut self,
        shader: &ShaderAsset,
        stage: &ShaderStage,
        range: &SurfaceRange,
        index_dynamic: bool,
        cpu: &[WorldVertex],
        assets: &RenderAssets,
        noise: &NoiseState,
        float_time: f32,
        surface_fog: Option<SurfaceFog>,
        entity: Option<&trRefEntity_t>,
        dynamic_vertices: &mut Vec<WorldVertex>,
        globals_offset: u32,
        refdef_time: i32,
        view_axis: [f32; 3],
    ) -> Option<StageDrawItem> {
        // A surface-sprite stage draws no plain geometry. The sprite chain is a
        // later wave, so the stage is skipped whole, as the oracle does.
        if let Some(ss) = &stage.ss {
            if ss.surfaceSpriteType != 0 {
                self.warn_once(Warned::SurfaceSprite);
                return None;
            }
        }

        let time = StageTime::new(float_time, shader.time_offset);

        // The current entity's renderfx overrides for this stage. Two of them
        // change the state bits, so they resolve before the pipeline key.
        // Source: oracle/codemp/renderer/tr_shade.cpp:2039-2054,2190-2202
        let fx = EntityFx::resolve(entity);
        let depth_range = DepthRange::resolve(entity);
        let state_bits = fx.state_bits(stage.state_bits);
        let alpha_func = alpha_func_code(state_bits);
        let key = PipelineKey {
            blend: blend_state_from_gls(state_bits),
            depth_equal: (state_bits & GLS_DEPTHFUNC_EQUAL as u32) != 0,
            depth_write: (state_bits & GLS_DEPTHMASK_TRUE as u32) != 0,
        };

        // The `ComputeColors` tail modulates the stage colours by fog density
        // when the surface is fogged and the stage sets `adjustColorsForFog`.
        // That per-frame math cannot bake into the static buffer, so a fogged
        // modulating stage always routes through the dynamic path.
        // Source: oracle/codemp/renderer/tr_shade.cpp:1509-1526 (the ACFF switch)
        let fog_mod = surface_fog.filter(|_| stage.adjust_colors_for_fog != acff_t::ACFF_NONE);

        // These stage kinds still draw as a plain stage, but each logs once so
        // the missing behavior stays visible.
        if fx.distortion {
            self.warn_once(Warned::Distortion);
        }
        if stage.glow {
            self.warn_once(Warned::Glow);
        }
        let bundle0 = &stage.bundle[0];
        if bundle0.is_video_map {
            self.warn_once(Warned::VideoMap);
        }

        let modulate = is_modulate_collapse(shader, stage);

        // A second bundle under any other env has no collapse path here, so it
        // draws bundle 0 alone.
        if stage.bundle[1].image.is_some() && !modulate {
            self.warn_once(Warned::MultitexEnv);
        }

        if modulate {
            // Two-texture pass: bundle 0 times the lightmap, the per-vertex
            // colour ignored, so only bundle 0's texcoords can force a dynamic
            // block. `is_modulate_collapse` guarantees bundle 1 is the lightmap,
            // and texture unit 1 reads `lightmap_st`. A bundle-1 tcMod has no
            // effect through this port, so it logs once.
            if !stage.bundle[1].tex_mods.is_empty() {
                self.warn_once(Warned::MultitexEnv);
            }
            let diffuse = stage_image(bundle0, time.shader_time);
            let lightmap = stage_image(&stage.bundle[1], time.shader_time);
            // The collapsed two-texture shader ignores the vertex color, so
            // fog modulation cannot reach the screen here and is dropped.
            // ACFF needs blend bits a collapsed stage does not carry, so this
            // arm is near unreachable with fog.
            let dynamic = !bundle0.tex_mods.is_empty() || fx.rewrites_colors();
            let base_vertex = if dynamic {
                let (source, _) = st_source(bundle0);
                build_dynamic_block(
                    cpu,
                    stage,
                    source,
                    true,
                    time,
                    noise,
                    assets,
                    &shader.name,
                    None,
                    entity,
                    // The world path drops the BSP normal this wave, so a
                    // lighting-diffuse stage here keeps the input colour.
                    None,
                    &mut self.stage_warnings,
                    dynamic_vertices,
                    fx,
                    refdef_time,
                    view_axis,
                )
            } else {
                range.base_vertex
            };
            return Some(StageDrawItem {
                key,
                diffuse,
                lightmap,
                mode: MODE_MULTITEXTURE,
                tex_from_lightmap: false,
                alpha_func,
                reads_lightmap: true,
                first_index: range.first_index,
                index_count: range.index_count,
                base_vertex,
                dynamic,
                index_dynamic,
                globals_offset,
                depth_far: false,
                depth_range,
            });
        }

        // Single-texture pass.
        let (source, unsupported) = st_source(bundle0);
        if unsupported {
            self.warn_once(Warned::TcGen);
        }
        let reads_lightmap = source == StSource::Lightmap;
        let diffuse = stage_image(bundle0, time.shader_time);
        let dynamic = stage_is_dynamic(stage, false) || fog_mod.is_some() || fx.rewrites_colors();

        if dynamic {
            let base_vertex = build_dynamic_block(
                cpu,
                stage,
                source,
                false,
                time,
                noise,
                assets,
                &shader.name,
                fog_mod,
                entity,
                // The world path drops the BSP normal this wave, so a
                // lighting-diffuse stage here keeps the input colour.
                None,
                &mut self.stage_warnings,
                dynamic_vertices,
                fx,
                refdef_time,
                view_axis,
            );
            Some(StageDrawItem {
                key,
                diffuse,
                lightmap: None,
                mode: MODE_SINGLE,
                // The dynamic block already holds the resolved texcoords in the
                // `st` field, so the shader reads `st` directly.
                tex_from_lightmap: false,
                alpha_func,
                reads_lightmap,
                first_index: range.first_index,
                index_count: range.index_count,
                base_vertex,
                dynamic: true,
                index_dynamic,
                globals_offset,
                depth_far: false,
                depth_range,
            })
        } else {
            Some(StageDrawItem {
                key,
                diffuse,
                lightmap: None,
                mode: MODE_SINGLE,
                tex_from_lightmap: reads_lightmap,
                alpha_func,
                reads_lightmap,
                first_index: range.first_index,
                index_count: range.index_count,
                base_vertex: range.base_vertex,
                dynamic: false,
                index_dynamic,
                globals_offset,
                depth_far: false,
                depth_range,
            })
        }
    }

    /// Resolves one active stage of a CPU-decoded entity surface (MD3 or Ghoul2)
    /// into its draw pass. These surfaces live only on the CPU - MD3 is
    /// keyframe-lerped, Ghoul2 is bone-deformed - so every stage builds a
    /// per-stage dynamic vertex block from the decoded surface, runs the stage's
    /// texcoord and colour evaluators over it, and draws it single-textured. The
    /// shared index block already sits in the per-frame index buffer.
    ///
    /// A surface-sprite stage yields `None`, the same skip `build_stage_item`
    /// makes for the world path.
    ///
    /// Source: `oracle/codemp/renderer/tr_shade.cpp:2136-2158`
    #[allow(clippy::too_many_arguments)]
    fn build_cpu_surface_stage_item(
        &mut self,
        shader: &ShaderAsset,
        stage: &ShaderStage,
        cpu_vertices: &[WorldVertex],
        first_index: u32,
        index_count: u32,
        float_time: f32,
        noise: &NoiseState,
        assets: &RenderAssets,
        surface_fog: Option<SurfaceFog>,
        entity: Option<&trRefEntity_t>,
        normals: Option<&[[f32; 4]]>,
        dynamic_vertices: &mut Vec<WorldVertex>,
        globals_offset: u32,
        refdef_time: i32,
        view_axis: [f32; 3],
    ) -> Option<StageDrawItem> {
        // A surface-sprite stage draws no plain geometry.
        if let Some(ss) = &stage.ss {
            if ss.surfaceSpriteType != 0 {
                self.warn_once(Warned::SurfaceSprite);
                return None;
            }
        }

        let time = StageTime::new(float_time, shader.time_offset);

        // The current entity's renderfx overrides for this stage. Two of them
        // change the state bits, so they resolve before the pipeline key.
        // Source: oracle/codemp/renderer/tr_shade.cpp:2039-2054,2190-2202
        let fx = EntityFx::resolve(entity);
        let depth_range = DepthRange::resolve(entity);
        if fx.distortion {
            self.warn_once(Warned::Distortion);
        }
        let state_bits = fx.state_bits(stage.state_bits);
        let alpha_func = alpha_func_code(state_bits);
        let key = PipelineKey {
            blend: blend_state_from_gls(state_bits),
            depth_equal: (state_bits & GLS_DEPTHFUNC_EQUAL as u32) != 0,
            depth_write: (state_bits & GLS_DEPTHMASK_TRUE as u32) != 0,
        };

        // The fog-density colour modulation runs on the per-frame block when the
        // surface is fogged and the stage sets `adjustColorsForFog`.
        // Source: oracle/codemp/renderer/tr_shade.cpp:1783-1800 (the ACFF switch)
        let fog_mod = surface_fog.filter(|_| stage.adjust_colors_for_fog != acff_t::ACFF_NONE);

        if stage.glow {
            self.warn_once(Warned::Glow);
        }
        let bundle0 = &stage.bundle[0];
        if bundle0.is_video_map {
            self.warn_once(Warned::VideoMap);
        }
        if stage.bundle[1].image.is_some() {
            self.warn_once(Warned::MultitexEnv);
        }

        let (source, unsupported) = st_source(bundle0);
        if unsupported {
            self.warn_once(Warned::TcGen);
        }
        let diffuse = stage_image(bundle0, time.shader_time);

        // The MD3 vertices are CPU-only, so every stage builds its own block in
        // the dynamic vertex buffer.
        let base_vertex = build_dynamic_block(
            cpu_vertices,
            stage,
            source,
            false,
            time,
            noise,
            assets,
            &shader.name,
            fog_mod,
            entity,
            normals,
            &mut self.stage_warnings,
            dynamic_vertices,
            fx,
            refdef_time,
            view_axis,
        );

        Some(StageDrawItem {
            key,
            diffuse,
            lightmap: None,
            mode: MODE_SINGLE,
            tex_from_lightmap: false,
            alpha_func,
            reads_lightmap: false,
            first_index,
            index_count,
            base_vertex,
            dynamic: true,
            index_dynamic: true,
            globals_offset,
            depth_far: false,
            depth_range,
        })
    }

    /// Builds the fog pass over one surface's geometry (`RB_FogPass`): every
    /// vertex takes the fog volume's packed colour and the fog texcoords, and
    /// the pass draws the fog image with the fog blend state. The vertices go
    /// into the per-frame dynamic buffer, since the fog colour and texcoords
    /// are per-frame. The indices come from the caller's own block - the static
    /// world index buffer for a world surface, or the per-frame index buffer
    /// for an MD3 or Ghoul2 surface.
    ///
    /// The blend is `GLS_SRCBLEND_SRC_ALPHA | GLS_DSTBLEND_ONE_MINUS_SRC_ALPHA`,
    /// with `GLS_DEPTHFUNC_EQUAL` when the shader's `fogPass` is `FP_EQUAL`.
    /// Depth writes stay off, the same as the oracle's fog `GL_State`.
    ///
    /// Source: `oracle/codemp/renderer/tr_shade.cpp:1182-1209`
    #[allow(clippy::too_many_arguments)]
    fn build_fog_stage_item(
        &mut self,
        cpu: &[WorldVertex],
        surface_fog: SurfaceFog,
        fog_pass: FogPass,
        fog_image: Option<ImageHandle>,
        first_index: u32,
        index_count: u32,
        index_dynamic: bool,
        globals_offset: u32,
        dynamic_vertices: &mut Vec<WorldVertex>,
        depth_range: DepthRange,
    ) -> Option<StageDrawItem> {
        // A missing fog image would bind the white fallback and paint the
        // surface at full fog density, so the pass skips instead.
        if fog_image.is_none() {
            self.warn_once(Warned::FogImageMissing);
            return None;
        }

        // `RB_CalcFogTexCoords` reads `tess.xyz`, a vec4 whose w is unused, so
        // the position widens to `[x, y, z, 0]`.
        let xyz: Vec<[f32; 4]> = cpu
            .iter()
            .map(|v| [v.position[0], v.position[1], v.position[2], 0.0])
            .collect();
        let data = RB_FogPass(&xyz, surface_fog.fog, surface_fog.ori, surface_fog.view_ori);

        let base_vertex = dynamic_vertices.len() as i32;
        for (i, v) in cpu.iter().enumerate() {
            dynamic_vertices.push(WorldVertex {
                position: v.position,
                st: data.tex_coords[i],
                lightmap_st: data.tex_coords[i],
                color: data.colors[i],
            });
        }

        let state_bits = (GLS_SRCBLEND_SRC_ALPHA | GLS_DSTBLEND_ONE_MINUS_SRC_ALPHA) as u32;
        let key = PipelineKey {
            blend: blend_state_from_gls(state_bits),
            depth_equal: fog_pass == FogPass::Equal,
            depth_write: false,
        };

        Some(StageDrawItem {
            key,
            diffuse: fog_image,
            lightmap: None,
            mode: MODE_SINGLE,
            tex_from_lightmap: false,
            alpha_func: 0,
            reads_lightmap: false,
            first_index,
            index_count,
            base_vertex,
            dynamic: true,
            index_dynamic,
            globals_offset,
            depth_far: false,
            depth_range,
        })
    }

    /// Builds and caches the pipeline for `key` if it is not already there.
    fn ensure_pipeline(&mut self, gpu: &Gpu, key: PipelineKey) {
        if self.pipelines.contains_key(&key) {
            return;
        }
        let pipeline = self.build_world_pipeline(
            gpu,
            key,
            &self.pipeline_layout,
            &self.shader,
            "mp_renderer_gpu world pipeline",
        );
        self.pipelines.insert(key, pipeline);
    }

    /// Builds the PBR pipeline for `key` into the PBR cache if it is not there
    /// yet (DEC-37 ruling 5, backend #2). It uses the same key, blend, and depth
    /// state the faithful pipeline uses, so a `Pbr` frame draws the same
    /// surfaces with the same pass order. Only the shader and the group-1 layout
    /// differ.
    fn ensure_pbr_pipeline(&mut self, gpu: &Gpu, key: PipelineKey) {
        if self.pbr_pipelines.contains_key(&key) {
            return;
        }
        let pipeline = self.build_world_pipeline(
            gpu,
            key,
            &self.pbr_pipeline_layout,
            &self.pbr_shader,
            "mp_renderer_gpu pbr world pipeline",
        );
        self.pbr_pipelines.insert(key, pipeline);
    }

    /// Builds one world render pipeline for `key` against `layout` and `shader`.
    /// The faithful and PBR backends share this builder, so the two pipelines
    /// agree on blend, depth, cull, and vertex layout by construction. The
    /// faithful path passes the faithful layout and shader, so its pipeline is
    /// bit-identical to the pre-seam pipeline.
    fn build_world_pipeline(
        &self,
        gpu: &Gpu,
        key: PipelineKey,
        layout: &wgpu::PipelineLayout,
        shader: &wgpu::ShaderModule,
        label: &str,
    ) -> RenderPipeline {
        let depth_compare = if key.depth_equal {
            wgpu::CompareFunction::Equal
        } else {
            wgpu::CompareFunction::LessEqual
        };

        gpu.device()
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(layout),
                vertex: wgpu::VertexState {
                    module: shader,
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
                    module: shader,
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
            })
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

    /// Grows the globals buffer (and its bind group) when `needed` clip-matrix
    /// slots exceed capacity. The buffer is reused across frames, so it only
    /// ever grows.
    fn reserve_globals(&mut self, gpu: &Gpu, needed: usize) {
        if needed <= self.globals_capacity {
            return;
        }
        let mut capacity = self.globals_capacity.max(1);
        while capacity < needed {
            capacity *= 2;
        }
        self.globals_buffer = create_globals_buffer(gpu.device(), capacity);
        self.globals_bind_group =
            create_globals_bind_group(gpu.device(), &self.globals_layout, &self.globals_buffer);
        self.globals_capacity = capacity;
    }

    /// Writes one clip matrix per slot into the globals buffer, each at its own
    /// stride slot so the dynamic offset lands on it. Slot 0 is the world.
    fn write_globals(&self, gpu: &Gpu, clips: &[[f32; 16]]) {
        let mut bytes = vec![0u8; clips.len() * GLOBALS_STRIDE as usize];
        for (slot, clip) in clips.iter().enumerate() {
            let offset = slot * GLOBALS_STRIDE as usize;
            let src = bytemuck::bytes_of(clip);
            bytes[offset..offset + src.len()].copy_from_slice(src);
        }
        gpu.queue().write_buffer(&self.globals_buffer, 0, &bytes);
    }

    /// Grows the per-frame dynamic vertex buffer when `needed` vertices exceed
    /// its capacity. The buffer is reused across frames, so it only ever grows.
    fn reserve_dynamic(&mut self, gpu: &Gpu, needed: usize) {
        if needed <= self.dynamic_capacity {
            return;
        }
        let mut capacity = self.dynamic_capacity.max(1);
        while capacity < needed {
            capacity *= 2;
        }
        self.dynamic_buffer = create_dynamic_buffer(gpu.device(), capacity);
        self.dynamic_capacity = capacity;
    }

    /// Grows the per-frame dynamic index buffer when `needed` indices exceed its
    /// capacity. The buffer is reused across frames, so it only ever grows.
    fn reserve_dynamic_indices(&mut self, gpu: &Gpu, needed: usize) {
        if needed <= self.dynamic_index_capacity {
            return;
        }
        let mut capacity = self.dynamic_index_capacity.max(1);
        while capacity < needed {
            capacity *= 2;
        }
        self.dynamic_index_buffer = create_dynamic_index_buffer(gpu.device(), capacity);
        self.dynamic_index_capacity = capacity;
    }

    /// Writes one flags block per draw item into the flags buffer, each at its
    /// own stride slot so the dynamic offset lands on it. A `Faithful` frame
    /// writes `pbr_lit` as zero, so its uniform bytes match the pre-seam bytes.
    fn write_flags(&self, gpu: &Gpu, items: &[StageDrawItem], mode: BackendMode) {
        let mut bytes = vec![0u8; items.len() * SURFACE_FLAGS_STRIDE as usize];
        for (draw_index, item) in items.iter().enumerate() {
            let pbr_lit = match mode {
                BackendMode::Faithful => 0,
                BackendMode::Pbr => pbr_lit_flag(item) as u32,
            };
            let flags = SurfaceFlagsGpu {
                mode: item.mode,
                tex_from_lightmap: item.tex_from_lightmap as u32,
                alpha_func: item.alpha_func,
                pbr_lit,
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

/// Whether the PBR backend shades this stage (the routing rule). A stage takes
/// the lit path only when it draws opaque. The collapsed lightmapped base pass
/// is opaque, so every lit world stage on duel1 and ffa2 qualifies through this
/// test alone. An additive or blended effect stage (glow, sprite, blood, an
/// uncollapsed `$lightmap` multiply pass) draws through the faithful colour
/// path unchanged, since lighting an effect texture breaks it. A sky pass
/// (`depth_far`) always stays faithful, even though the sky box draws opaque,
/// because the sky is an effect surface, not a lit one.
fn pbr_lit_flag(item: &StageDrawItem) -> bool {
    !item.depth_far && item.key.blend == OPAQUE
}

/// Builds one clip matrix per distinct entity number the draw list carries,
/// the matching model orientation, and the entity-number-to-slot map. Slot 0 is
/// always the world (`view.world`). Each real entity's orientation comes from
/// `R_RotateForEntity` against its `trRefEntity_t` row and the view. The clip
/// matrix is `correction * projection * model` in every slot, and the fog pass
/// reads the orientation for `RB_CalcFogTexCoords`.
///
/// A draw surf whose decoded entity number is out of the `entities` slice falls
/// back to the world orientation, so a stale sort key can never index past the
/// row list.
///
/// Source: `oracle/codemp/renderer/tr_main.cpp:302-360` (`R_RotateForEntity`)
fn build_entity_slots(
    draw_surfs: &[DrawSurf<SurfaceGeometry>],
    assets: &RenderAssets,
    view: &viewParms_t,
    entities: &[trRefEntity_t],
    scratch: &mut TrMainScratch,
) -> (Vec<[f32; 16]>, Vec<orientationr_t>, HashMap<i32, u32>) {
    let projection = &view.projectionMatrix;
    let mut clips: Vec<[f32; 16]> = Vec::new();
    let mut oris: Vec<orientationr_t> = Vec::new();
    let mut slot_map: HashMap<i32, u32> = HashMap::new();

    // Slot 0 is the world, so the entity-free path keeps one aligned slot at
    // offset 0 with the plain view clip matrix and the world orientation.
    clips.push(world_clip_matrix(&view.world.modelMatrix, projection));
    oris.push(world_orientation(view));
    slot_map.insert(TR_WORLDENT, 0);

    for surf in draw_surfs {
        let (entity_num, _shader, _fog, _dlight) =
            R_DecomposeSort(surf.sort, &assets.sorted_shaders);
        if slot_map.contains_key(&entity_num) {
            continue;
        }

        let ori = match entities.get(entity_num as usize) {
            Some(ent) => R_RotateForEntity(ent, view, scratch),
            None => world_orientation(view),
        };
        let slot = clips.len() as u32;
        clips.push(world_clip_matrix(&ori.modelMatrix, projection));
        oris.push(ori);
        slot_map.insert(entity_num, slot);
    }

    (clips, oris, slot_map)
}

/// The world model orientation (`viewParms->world`), the value the backend
/// leaves in `backEnd.ori` for a world surface. `orientationr_t` carries no
/// `Clone`, so this copies its fields explicitly.
fn world_orientation(view: &viewParms_t) -> orientationr_t {
    orientationr_t {
        origin: view.world.origin,
        axis: view.world.axis,
        viewOrigin: view.world.viewOrigin,
        modelMatrix: view.world.modelMatrix,
    }
}

/// The fog inputs for a surface whose sort key decoded to `fog_num`. Fog number
/// 0 means no fog, so this returns `None`. Otherwise it resolves the fog volume
/// from `world.fogs`, pairs it with the surface's slot orientation and the
/// camera orientation, and hands back the [`SurfaceFog`] the stage build reads.
///
/// An out-of-range fog number drops to `None` rather than index past the fog
/// list, the same defensive skip a stale sort key gets elsewhere.
fn resolve_surface_fog<'a>(
    fog_num: i32,
    fogs: &'a [fog_t],
    oris: &'a [orientationr_t],
    slot: u32,
    view: &'a viewParms_t,
) -> Option<SurfaceFog<'a>> {
    if fog_num == 0 {
        return None;
    }
    let fog = fogs.get(fog_num as usize)?;
    let ori = oris.get(slot as usize)?;
    Some(SurfaceFog {
        fog,
        ori,
        view_ori: &view.ori,
    })
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

/// The bezier-patch grid at flat surface `index`, or `None` when the surface is
/// another kind or no world is loaded. The surface array is in the same lump
/// order the geometry ranges were built from, so `index` addresses both.
fn world_surface_grid(assets: &RenderAssets, index: u32) -> Option<&GridMesh> {
    match &assets.world.as_ref()?.surfaces.get(index as usize)?.data {
        SurfaceData::Grid(grid) => Some(grid),
        _ => None,
    }
}

/// The identity-light white the sky box draws with (`qglColor3f(tr.identityLight,
/// tr.identityLight, tr.identityLight)`, alpha 1). The world shader multiplies
/// the face texel by this per-vertex colour.
///
/// Source: `oracle/codemp/renderer/tr_sky.cpp:820`
fn sky_identity_color() -> [u8; 4] {
    let c = (255.0 * IDENTITY_LIGHT) as u8;
    [c, c, c, 255]
}

/// Builds one outer-box face's grid vertices and triangle indices into the
/// per-frame dynamic buffers, returning `(first_index, index_count, base_vertex)`.
/// `DrawSkySide` emits one `GL_TRIANGLE_STRIP` per grid row over the face's
/// subdivision bounds. This converts each row strip into an indexed triangle
/// list, keeping the strip vertex order. The face points are view-origin
/// relative, so each vertex adds the view origin (the oracle's `qglTranslatef`).
///
/// Source: `oracle/codemp/renderer/tr_sky.cpp:347-376`
fn build_sky_face_block(
    face: &SkyBoxFace,
    view_origin: vec3_t,
    color: [u8; 4],
    dynamic_vertices: &mut Vec<WorldVertex>,
    dynamic_indices: &mut Vec<u32>,
) -> (u32, u32, i32) {
    let base_vertex = dynamic_vertices.len() as i32;
    let first_index = dynamic_indices.len() as u32;
    let half = HALF_SKY_SUBDIVISIONS;

    for t in (face.mins[1] + half)..(face.maxs[1] + half) {
        // The strip's first vertex, local to this face's own vertex block.
        let strip_start = dynamic_vertices.len() as u32 - base_vertex as u32;
        let mut strip_len = 0u32;

        for s in (face.mins[0] + half)..=(face.maxs[0] + half) {
            let s = s as usize;
            for row in [t as usize, (t + 1) as usize] {
                let p = face.points[row][s];
                let st = face.tex_coords[row][s];
                dynamic_vertices.push(WorldVertex {
                    position: [
                        p[0] + view_origin[0],
                        p[1] + view_origin[1],
                        p[2] + view_origin[2],
                    ],
                    st,
                    lightmap_st: st,
                    color,
                });
            }
            strip_len += 2;
        }

        // A triangle strip of N vertices makes N - 2 triangles, each (i, i+1,
        // i+2). The pipeline culls no faces, so the strip's own winding carries
        // straight into the list.
        for i in 0..strip_len.saturating_sub(2) {
            dynamic_indices.push(strip_start + i);
            dynamic_indices.push(strip_start + i + 1);
            dynamic_indices.push(strip_start + i + 2);
        }
    }

    let index_count = dynamic_indices.len() as u32 - first_index;
    (first_index, index_count, base_vertex)
}

/// Which vertex texcoord a world stage's bundle reads before any tcMod.
///
/// - `Base`: the surface's own `st`.
/// - `Lightmap`: the surface's lightmap `st`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StSource {
    Base,
    Lightmap,
}

/// The texcoord source a bundle's `tcGen` selects, plus whether that `tcGen`
/// has no world path yet. `TCGEN_LIGHTMAP` reads the lightmap st, `TCGEN_TEXTURE`
/// reads the base st, and every other kind reads the base st and reports the gap.
///
/// The generator switch here is a second copy of the one `stage2d::stage_texcoords`
/// owns, and the two disagree on `TCGEN_IDENTITY`: `stage_texcoords` zeroes the
/// st, this reads the base st. The world path picks the base or lightmap source
/// before it runs the tcMod loop, and it has no zero source yet, so the two
/// copies stay split until a `TCGEN_IDENTITY` world stage needs the zero.
///
/// Source: `oracle/codemp/renderer/tr_shade.cpp:1820-1854` (`ComputeTexCoords`)
//TODO: Port TCGEN_IDENTITY world path (unify the generator switch into stage2d)
// Source: oracle/codemp/renderer/tr_shade.cpp:1809-1854
fn st_source(bundle: &TextureBundle) -> (StSource, bool) {
    match bundle.tc_gen {
        texCoordGen_t::TCGEN_LIGHTMAP => (StSource::Lightmap, false),
        texCoordGen_t::TCGEN_TEXTURE => (StSource::Base, false),
        _ => (StSource::Base, true),
    }
}

/// Whether a stage needs a per-frame dynamic vertex block rather than the static
/// world buffer. A two-texture collapse ignores the vertex colour, so only its
/// bundle 0 tcMods can force a dynamic block. A single-texture stage also needs
/// a dynamic block when its colour gen is not a plain vertex pass-through, since
/// `ComputeColors` then writes a new colour the static buffer does not hold.
fn stage_is_dynamic(stage: &ShaderStage, two_texture: bool) -> bool {
    let has_tex_mods = !stage.bundle[0].tex_mods.is_empty();
    if two_texture {
        return has_tex_mods;
    }
    has_tex_mods || !stage_colors_are_vertex_passthrough(stage)
}

/// Whether the stage's rgbGen and alphaGen pass the BSP vertex colour through
/// unchanged. Only then does the static buffer's own colour serve the stage.
/// `CGEN_IDENTITY` writes white, `CGEN_CONST` writes the constant, and every
/// other gen writes something new, so those stages need `ComputeColors` on a
/// per-frame dynamic block. `CGEN_EXACT_VERTEX` and `CGEN_VERTEX` (at identity
/// light) keep the vertex colour, and `AGEN_SKIP`/`AGEN_VERTEX` keep the vertex
/// alpha.
///
/// Source: `oracle/codemp/renderer/tr_shade.cpp:1591-1779` (`ComputeColors`)
fn stage_colors_are_vertex_passthrough(stage: &ShaderStage) -> bool {
    let rgb_passthrough = stage.rgb_gen == colorGen_t::CGEN_EXACT_VERTEX
        || (stage.rgb_gen == colorGen_t::CGEN_VERTEX && IDENTITY_LIGHT == 1.0);
    let alpha_passthrough = matches!(
        stage.alpha_gen,
        alphaGen_t::AGEN_SKIP | alphaGen_t::AGEN_VERTEX
    );
    rgb_passthrough && alpha_passthrough
}

/// The shader's `GLS_ATEST` alpha-test compare, as the code the world shader
/// discards by: 0 none, 1 `GT_0`, 2 `LT_80`, 3 `GE_80`, 4 `GE_C0`. Alpha test
/// is a per-fragment discard the shader drives through the flags uniform, so it
/// stays out of [`PipelineKey`].
///
/// Source: `oracle/codemp/renderer/tr_local.h` (`GLS_ATEST_*`), `GL_State`
fn alpha_func_code(state_bits: u32) -> u32 {
    if state_bits & GLS_ATEST_GT_0 != 0 {
        1
    } else if state_bits & GLS_ATEST_LT_80 != 0 {
        2
    } else if state_bits & GLS_ATEST_GE_80 != 0 {
        3
    } else if state_bits & GLS_ATEST_GE_C0 != 0 {
        4
    } else {
        0
    }
}

/// Whether the shader collapses bundle 0 and bundle 1 into one modulated pass.
/// Only `GL_MODULATE` over a lightmap bundle 1 keeps the two-texture pass. The
/// oracle draws it as diffuse times lightmap in one `DrawMultitextured` call,
/// and texture unit 1 reads the lightmap st.
///
/// Source: `oracle/codemp/renderer/tr_shade.cpp:2136-2140`
fn is_modulate_collapse(shader: &ShaderAsset, stage: &ShaderStage) -> bool {
    shader.multitexture_env == GL_MODULATE
        && stage.bundle[1].image.is_some()
        && stage.bundle[1].is_lightmap
}

/// Builds the small [`RefEntity`] the diffuse-lighting evaluators read from a
/// `trRefEntity_t`. `R_SetupEntityLighting` folded the entity light into
/// `lightDir`/`ambientLight`/`directedLight`, and the evaluators read only
/// those plus `shaderRGBA`, so the rest stays at its default.
fn lighting_ref_entity(ent: &trRefEntity_t) -> RefEntity {
    RefEntity {
        light_dir: ent.lightDir,
        ambient_light: ent.ambientLight,
        directed_light: ent.directedLight,
        shader_rgba: ent.e.shaderRGBA,
        ..Default::default()
    }
}

/// Evaluates one stage's per-vertex texcoords and colours for `cpu` and appends
/// the result to `out`, returning the block's base vertex in the dynamic buffer.
/// A two-texture collapse copies the vertex colour through unchanged, since its
/// shader path ignores it.
///
/// `entity` and `normals` are the CPU-surface lighting context. A
/// lighting-diffuse stage reads them: `normals` is the parallel decoded normal
/// slice, and `entity` carries the light `R_SetupEntityLighting` folded in.
/// The world and sky paths pass `None`, so a lighting-diffuse stage on a
/// surface kind with no normals keeps the input colour.
///
/// Source: `oracle/codemp/renderer/tr_shade.cpp:2117-2118` (`ComputeColors`
/// then `ComputeTexCoords` per stage)
#[allow(clippy::too_many_arguments)]
fn build_dynamic_block(
    cpu: &[WorldVertex],
    stage: &ShaderStage,
    source: StSource,
    two_texture: bool,
    time: StageTime,
    noise: &NoiseState,
    assets: &RenderAssets,
    shader_name: &str,
    fog_mod: Option<SurfaceFog>,
    entity: Option<&trRefEntity_t>,
    normals: Option<&[[f32; 4]]>,
    warnings: &mut Stage2dWarnings,
    out: &mut Vec<WorldVertex>,
    fx: EntityFx,
    refdef_time: i32,
    view_axis: [f32; 3],
) -> i32 {
    let base_vertex = out.len() as i32;
    let count = cpu.len();

    // Seed the texcoords from the chosen source, then run the tcMod loop. The
    // `TMOD_ENTITY_TRANSLATE` arm scrolls by the entity's `shaderTexCoord`.
    let mut st: Vec<[f32; 2]> = cpu
        .iter()
        .map(|v| match source {
            StSource::Base => v.st,
            StSource::Lightmap => v.lightmap_st,
        })
        .collect();
    apply_tex_mods(
        &stage.bundle[0],
        &mut st,
        time,
        noise,
        assets,
        shader_name,
        entity.map(|ent| ent.e.shaderTexCoord),
        warnings,
    );

    // The `xyz` block the disintegrate and volumetric arms read, and the one
    // `RB_CalcDisintegrateVertDeform` moves.
    let mut xyz: Vec<[f32; 4]> = cpu
        .iter()
        .map(|v| [v.position[0], v.position[1], v.position[2], 0.0])
        .collect();

    // `ComputeColors`'s two short-circuits: either one produces the whole
    // colour block and skips both generator switches, and the disintegrate arm
    // additionally moves the vertices.
    // Source: oracle/codemp/renderer/tr_shade.cpp:1535-1581
    let short_circuit = (fx.disintegrate1 || fx.disintegrate2 || fx.volumetric)
        .then(|| entity.map(lighting_ref_entity))
        .flatten();

    // A single-texture stage runs the full rgbGen/alphaGen. A two-texture
    // collapse keeps the input colour, which its shader path never reads.
    let mut colors: Vec<[u8; 4]> = if let Some(re) = short_circuit {
        let mut evaluated = vec![[0u8; 4]; count];
        if fx.disintegrate1 || fx.disintegrate2 {
            RB_CalcDisintegrateColors(&mut evaluated, &xyz, &re, refdef_time);
            // The vert deform only fires under `RF_DISINTEGRATE2`, and it needs
            // the surface normals. A world surface has none, so it keeps its
            // vertices.
            if let Some(n) = normals {
                RB_CalcDisintegrateVertDeform(&mut xyz, n, &re, refdef_time);
            }
        } else {
            volumetric_colors(&mut evaluated, normals, &re, view_axis);
        }
        evaluated
    } else if two_texture {
        cpu.iter().map(|v| v.color).collect()
    } else {
        let input: Vec<[u8; 4]> = cpu.iter().map(|v| v.color).collect();
        let mut evaluated = vec![[0u8; 4]; count];
        // `RF_RGB_TINT` forces `CGEN_ENTITY`, so the stage takes the entity's
        // own colour instead of its shader's rgbGen.
        // Source: oracle/codemp/renderer/tr_shade.cpp:2049-2053
        let rgb_gen = if fx.rgb_tint {
            colorGen_t::CGEN_ENTITY
        } else {
            stage.rgb_gen
        };
        let entity_view = entity.map(lighting_ref_entity);
        match (rgb_gen, normals, entity) {
            // The lighting-diffuse rgbGen runs first, then the alphaGen switch,
            // the same order the oracle's `ComputeColors` keeps.
            (colorGen_t::CGEN_LIGHTING_DIFFUSE, Some(n), Some(ent)) => {
                let re = lighting_ref_entity(ent);
                RB_CalcDiffuseColor(&mut evaluated, n, &re, ent.ambientLightInt);
                apply_alpha_gen(
                    stage,
                    &input,
                    &mut evaluated,
                    time,
                    noise,
                    assets,
                    shader_name,
                    warnings,
                );
            }
            (colorGen_t::CGEN_LIGHTING_DIFFUSE_ENTITY, Some(n), Some(ent)) => {
                let re = lighting_ref_entity(ent);
                RB_CalcDiffuseEntityColor(&mut evaluated, n, &re);
                apply_alpha_gen(
                    stage,
                    &input,
                    &mut evaluated,
                    time,
                    noise,
                    assets,
                    shader_name,
                    warnings,
                );
            }
            // Every other rgbGen, and a lighting-diffuse stage on a surface with
            // no decoded normals (world and sky), runs the shared switch. That
            // switch warns once for a lighting-diffuse rgbGen it cannot light.
            _ => {
                stage_colors_into(
                    stage,
                    rgb_gen,
                    entity_view.as_ref(),
                    &input,
                    &mut evaluated,
                    time,
                    noise,
                    assets,
                    shader_name,
                    warnings,
                );
            }
        }
        evaluated
    };

    // `ForceAlpha`: every vertex takes the entity's own alpha. It runs after
    // the generators and before the fog tail, the same place the oracle's
    // `else if` arm sits in the stage loop.
    // Source: oracle/codemp/renderer/tr_shade.cpp:2190-2192
    if fx.force_ent_alpha {
        for color in colors.iter_mut() {
            color[3] = fx.ent_alpha;
        }
    }

    // The `ComputeColors` fog tail: when the surface is fogged and the stage
    // sets `adjustColorsForFog`, fade the colours by fog density. The caller
    // hands `fog_mod` only when the stage's ACFF is not `ACFF_NONE`.
    // Source: oracle/codemp/renderer/tr_shade.cpp:1509-1526
    if let Some(sf) = fog_mod {
        match stage.adjust_colors_for_fog {
            acff_t::ACFF_MODULATE_RGB => {
                RB_CalcModulateColorsByFog(&mut colors, &xyz, sf.fog, sf.ori, sf.view_ori, assets)
            }
            acff_t::ACFF_MODULATE_ALPHA => {
                RB_CalcModulateAlphasByFog(&mut colors, &xyz, sf.fog, sf.ori, sf.view_ori, assets)
            }
            acff_t::ACFF_MODULATE_RGBA => {
                RB_CalcModulateRGBAsByFog(&mut colors, &xyz, sf.fog, sf.ori, sf.view_ori, assets)
            }
            acff_t::ACFF_NONE => {}
        }
    }

    for i in 0..count {
        out.push(WorldVertex {
            position: [xyz[i][0], xyz[i][1], xyz[i][2]],
            st: st[i],
            lightmap_st: cpu[i].lightmap_st,
            color: colors[i],
        });
    }

    base_vertex
}

/// Raven's `RF_VOLUMETRIC` short-circuit in `ComputeColors`: every channel,
/// alpha included, takes the entity's red channel faded by the fourth power of
/// the vertex normal's dot with the view axis.
///
/// A surface with no decoded normals (world and sky) has nothing to fade by, so
/// it takes the entity colour flat.
///
/// Source: `oracle/codemp/renderer/tr_shade.cpp:1554-1581`
fn volumetric_colors(
    colors: &mut [[u8; 4]],
    normals: Option<&[[f32; 4]]>,
    ent: &RefEntity,
    view_axis: [f32; 3],
) {
    let base = ent.shader_rgba[0] as f32;
    for (i, color) in colors.iter_mut().enumerate() {
        let dot = match normals.and_then(|n| n.get(i)) {
            Some(n) => {
                let mut dot = n[0] * view_axis[0] + n[1] * view_axis[1] + n[2] * view_axis[2];
                dot *= dot * dot * dot;
                // so low, so just clamp it
                if dot < 0.2 {
                    0.0
                } else {
                    dot
                }
            }
            None => 0.0,
        };
        let value = myftol(base * (1.0 - dot)) as u8;
        *color = [value, value, value, value];
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

fn create_dynamic_buffer(device: &wgpu::Device, capacity: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("mp_renderer_gpu world dynamic vertex buffer"),
        size: capacity as u64 * size_of::<WorldVertex>() as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn create_dynamic_index_buffer(device: &wgpu::Device, capacity: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("mp_renderer_gpu world dynamic index buffer"),
        size: capacity as u64 * size_of::<u32>() as u64,
        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

/// The per-entity shader clock offset `floatTime - e.shaderTime` reads: the
/// entity's own `shaderTime`, or 0 for the world entity and any unmapped
/// number.
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:910-916`
fn entity_shader_time(entities: &[trRefEntity_t], entity_num: i32) -> f32 {
    entities
        .get(entity_num as usize)
        .map(|ent| ent.e.shaderTime)
        .unwrap_or(0.0)
}

/// The renderfx overrides the current entity applies to every stage of its
/// surface, resolved once per stage build.
///
/// Raven spreads these across `RB_IterateStagesGeneric` (the state-bit and
/// `forceRGBGen` overrides) and `ComputeColors` (the colour short-circuits).
/// They are collected here because two of them change the pipeline key, which
/// is decided before any colour work.
///
/// Source: `oracle/codemp/renderer/tr_shade.cpp:2039-2054,2190-2202`
/// (`RB_IterateStagesGeneric`), `oracle/codemp/renderer/tr_shade.cpp:1535-1581`
/// (`ComputeColors`)
#[derive(Clone, Copy, Default)]
struct EntityFx {
    /// `RF_DISINTEGRATE1`: the procedural hole-ripping colours, plus a fixed
    /// state-bit set that depth-writes and alpha-tests the hole.
    disintegrate1: bool,
    /// `RF_DISINTEGRATE2`: the same colour short-circuit, no state override.
    disintegrate2: bool,
    /// `RF_RGB_TINT`: force `CGEN_ENTITY`, so the stage takes the entity's own
    /// `shaderRGBA` instead of the shader's rgbGen.
    rgb_tint: bool,
    /// `RF_FORCE_ENT_ALPHA`: overwrite every vertex alpha with the entity's,
    /// and force the alpha-blend state.
    force_ent_alpha: bool,
    /// `RF_ALPHA_DEPTH`: the force-alpha state additionally depth-writes.
    alpha_depth: bool,
    /// `RF_VOLUMETRIC`: the fake volumetric shading short-circuit.
    volumetric: bool,
    /// `RF_DISTORTION`: the screen-capture refraction pass.
    ///
    //TODO: Port RB_IterateStagesGeneric's RF_DISTORTION arm
    // Source: oracle/codemp/renderer/tr_shade.cpp:2162-2168
    // The arm binds `tr.screenImage` and switches to two-sided culling, so it
    // needs a screen-capture pass (`RB_CaptureScreenImage`) that no wave has
    // built. The flag is read only to log once, and the stage draws plain.
    distortion: bool,
    /// The entity's `shaderRGBA[3]`, the alpha `RF_FORCE_ENT_ALPHA` writes.
    ent_alpha: u8,
}

impl EntityFx {
    /// Reads the flags off the surface's entity. A surface with no entity (the
    /// world) applies nothing.
    fn resolve(entity: Option<&trRefEntity_t>) -> EntityFx {
        let Some(ent) = entity else {
            return EntityFx::default();
        };
        let fx = ent.e.renderfx;
        EntityFx {
            disintegrate1: fx & RF_DISINTEGRATE1 != 0,
            disintegrate2: fx & RF_DISINTEGRATE2 != 0,
            rgb_tint: fx & RF_RGB_TINT != 0,
            force_ent_alpha: fx & RF_FORCE_ENT_ALPHA != 0,
            alpha_depth: fx & RF_ALPHA_DEPTH != 0,
            volumetric: fx & RF_VOLUMETRIC != 0,
            distortion: fx & RF_DISTORTION != 0,
            ent_alpha: ent.e.shaderRGBA[3],
        }
    }

    /// The stage's state bits after the two overrides that replace them
    /// outright. `RF_DISINTEGRATE1` wins where both apply, matching the
    /// oracle's order: it rewrites `stateBits` at the top of the stage loop,
    /// and the force-alpha arm is an `else if` chain below that never sees it.
    fn state_bits(&self, stage_bits: u32) -> u32 {
        if self.disintegrate1 {
            return (GLS_SRCBLEND_SRC_ALPHA | GLS_DSTBLEND_ONE_MINUS_SRC_ALPHA
                | GLS_DEPTHMASK_TRUE) as u32
                | GLS_ATEST_GE_C0;
        }
        if self.force_ent_alpha {
            let mut bits = (GLS_SRCBLEND_SRC_ALPHA | GLS_DSTBLEND_ONE_MINUS_SRC_ALPHA) as u32;
            if self.alpha_depth {
                bits |= GLS_DEPTHMASK_TRUE as u32;
            }
            return bits;
        }
        stage_bits
    }

    /// Whether any override rewrites the per-vertex colours, which forces the
    /// stage onto the per-frame dynamic block.
    fn rewrites_colors(&self) -> bool {
        self.disintegrate1 || self.disintegrate2 || self.rgb_tint || self.force_ent_alpha
            || self.volumetric
    }
}

/// Appends `RB_AddQuadStampExt`'s four corners and six indices for a quad
/// centred at `origin` and spanned by `left` and `up`.
///
/// The oracle writes the same constant colour into all four vertices and the
/// same view-facing normal; the normal is dropped here, as it is for every
/// other CPU-built surface in this module.
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:60-124`
fn add_quad_stamp_ext(
    verts: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
    origin: vec3_t,
    left: vec3_t,
    up: vec3_t,
    color: [u8; 4],
    s1: f32,
    t1: f32,
    s2: f32,
    t2: f32,
) {
    let ndx = verts.len() as u32;
    indices.extend_from_slice(&[ndx, ndx + 1, ndx + 3, ndx + 3, ndx + 1, ndx + 2]);

    let corners = [
        ([1.0f32, 1.0f32], [s1, t1]),
        ([-1.0, 1.0], [s2, t1]),
        ([-1.0, -1.0], [s2, t2]),
        ([1.0, -1.0], [s1, t2]),
    ];
    for ([left_sign, up_sign], st) in corners {
        verts.push(WorldVertex {
            position: [
                origin[0] + left_sign * left[0] + up_sign * up[0],
                origin[1] + left_sign * left[1] + up_sign * up[1],
                origin[2] + left_sign * left[2] + up_sign * up[2],
            ],
            st,
            lightmap_st: [0.0, 0.0],
            color,
        });
    }
}

/// Raven `DoSprite` - a screen-oriented quad of half-size `radius`, rotated by
/// `rotation` degrees around the view axis.
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:533-555`
fn do_sprite(
    verts: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
    origin: vec3_t,
    radius: f32,
    rotation: f32,
    color: [u8; 4],
    view: &viewParms_t,
) {
    // `M_PI * rotation / 180.0f` promotes to `f64` through the unsuffixed
    // `M_PI`, then truncates back at the assignment.
    let ang = (PI * rotation as f64 / 180.0) as f32;
    let s = ang.sin();
    let c = ang.cos();

    let mut left: vec3_t = [0.0; 3];
    VectorScale(view.ori.axis[1], c * radius, &mut left);
    VectorMA(left, -s * radius, view.ori.axis[2], &mut left);

    let mut up: vec3_t = [0.0; 3];
    VectorScale(view.ori.axis[2], c * radius, &mut up);
    VectorMA(up, s * radius, view.ori.axis[1], &mut up);

    if view.isMirror != 0 {
        VectorSubtract(vec3_origin, left, &mut left);
    }

    add_quad_stamp_ext(verts, indices, origin, left, up, color, 0.0, 0.0, 1.0, 1.0);
}

/// Raven `DoLine` - a view-facing quad spanning `start` to `end`, `spanWidth`
/// wide either side of `up`.
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:601-660`
fn do_line(
    verts: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
    start: vec3_t,
    end: vec3_t,
    up: vec3_t,
    span_width: f32,
    color: [u8; 4],
) {
    let vbase = verts.len() as u32;
    let span_width2 = -span_width;

    for (base, width, st) in [
        (start, span_width, [0.0f32, 0.0f32]),
        (start, span_width2, [1.0, 0.0]),
        (end, span_width, [0.0, 1.0]),
        (end, span_width2, [1.0, 1.0]),
    ] {
        let mut xyz: vec3_t = [0.0; 3];
        VectorMA(base, width, up, &mut xyz);
        verts.push(WorldVertex {
            position: xyz,
            st,
            lightmap_st: [0.0, 0.0],
            color,
        });
    }

    indices.extend_from_slice(&[vbase, vbase + 1, vbase + 2, vbase + 2, vbase + 1, vbase + 3]);
}

/// Builds one generated entity surface's world-space vertices and triangle
/// indices, the `RB_SurfaceEntity` dispatch restricted to the DEC-54 census
/// set: `RT_SPRITE`, `RT_LINE`, and `RT_SABER_GLOW`.
///
/// Every other `reType` returns empty, which the caller counts as a skip.
/// `RT_BEAM`, `RT_ORIENTED_QUAD`, `RT_ORIENTEDLINE`, `RT_ELECTRICITY`, and
/// `RT_CYLINDER` are census-complement fog, so they stay unbuilt rather than
/// guessed at.
//TODO: Port RB_SurfaceBeam
// Source: oracle/codemp/renderer/tr_surface.cpp:226-283
//TODO: Port RB_SurfaceOrientedQuad
// Source: oracle/codemp/renderer/tr_surface.cpp:173-215
//TODO: Port RB_SurfaceOrientedLine
// Source: oracle/codemp/renderer/tr_surface.cpp:786-806
//TODO: Port RB_SurfaceElectricity
// Source: oracle/codemp/renderer/tr_surface.cpp:1038-1090
//TODO: Port RB_SurfaceCylinder
// Source: oracle/codemp/renderer/tr_surface.cpp:854-985
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:1798-1870`
fn build_entity_geometry(e: &refEntity_t, view: &viewParms_t) -> (Vec<WorldVertex>, Vec<u32>) {
    let mut verts: Vec<WorldVertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let color = e.shaderRGBA;

    match e.reType {
        // `RB_SurfaceSprite`: the screen-oriented quad, with the rotation
        // branch the oracle splits out for `rotation == 0`.
        // Source: oracle/codemp/renderer/tr_surface.cpp:141-169
        refEntityType_t::RT_SPRITE => {
            let radius = e.radius;
            let (mut left, mut up): (vec3_t, vec3_t) = ([0.0; 3], [0.0; 3]);
            if e.rotation == 0.0 {
                VectorScale(view.ori.axis[1], radius, &mut left);
                VectorScale(view.ori.axis[2], radius, &mut up);
            } else {
                let ang = (PI * e.rotation as f64 / 180.0) as f32;
                let s = ang.sin();
                let c = ang.cos();

                VectorScale(view.ori.axis[1], c * radius, &mut left);
                VectorMA(left, -s * radius, view.ori.axis[2], &mut left);

                VectorScale(view.ori.axis[2], c * radius, &mut up);
                VectorMA(up, s * radius, view.ori.axis[1], &mut up);
            }
            if view.isMirror != 0 {
                VectorSubtract(vec3_origin, left, &mut left);
            }
            add_quad_stamp_ext(
                &mut verts,
                &mut indices,
                e.origin,
                left,
                up,
                color,
                0.0,
                0.0,
                1.0,
                1.0,
            );
        }

        // `RB_SurfaceLine`: the side vector is the cross product of the two
        // eye-to-endpoint vectors, so the quad always faces the camera.
        // Source: oracle/codemp/renderer/tr_surface.cpp:667-690
        refEntityType_t::RT_LINE => {
            let end = e.oldorigin;
            let start = e.origin;

            let mut v1: vec3_t = [0.0; 3];
            let mut v2: vec3_t = [0.0; 3];
            VectorSubtract(start, view.ori.origin, &mut v1);
            VectorSubtract(end, view.ori.origin, &mut v2);
            let mut right: vec3_t = [0.0; 3];
            CrossProduct(v1, v2, &mut right);
            VectorNormalize(&mut right);

            do_line(&mut verts, &mut indices, start, end, right, e.radius, color);
        }

        // `RB_SurfaceSaberGlow`: a run of sprites stepped up the blade, then
        // the hilt blob.
        //
        // The oracle's `e->radius += 0.017f` walks the *live* entity, so each
        // sprite is a step wider than the last, and the widened radius persists
        // into the next frame's submission. This build works on the local copy
        // the loop carries, so the widening inside one blade is reproduced and
        // the cross-frame leak is not: the frontend hands the backend a
        // by-value entity list per scene (DEC-50), so there is no shared object
        // to write back through.
        //
        // Source: oracle/codemp/renderer/tr_surface.cpp:560-580
        refEntityType_t::RT_SABER_GLOW => {
            let mut radius = e.radius;
            let mut i = e.saberLength;
            while i > 0.0 {
                let mut end: vec3_t = [0.0; 3];
                VectorMA(e.origin, i, e.axis[0], &mut end);
                do_sprite(&mut verts, &mut indices, end, radius, 0.0, color, view);
                // The oracle widens the radius inside the body, so the loop
                // step below reads the already-widened value.
                radius += 0.017;
                i -= radius * 0.65;
            }

            // Big hilt sprite
            //
            //TODO: Port RB_SurfaceSaberGlow's random hilt radius
            // Source: oracle/codemp/renderer/tr_surface.cpp:579
            // The oracle radius is `5.5f + random() * 0.25f`. `random()` is
            // `rand()/32768` off the ambient C generator, which the render
            // thread does not own and which no image golden can pin. The low
            // end of the same quarter-unit span stands in until a render-side
            // generator lands.
            do_sprite(&mut verts, &mut indices, e.origin, 5.5, 0.0, color, view);
        }

        _ => {}
    }

    (verts, indices)
}

/// Decodes one MD3 (`MOD_MESH`) surface into GPU vertices and triangle indices.
/// The vertices are keyframe-lerped on the CPU (`LerpMeshVertexes`): the packed
/// shorts scale by `MD3_XYZ_SCALE`, blended by `backlerp` between the old and
/// new frame blocks. The texcoords come from `ofsSt`, the indices from
/// `ofsTriangles`, and the vertex colour is the entity's `shaderRGBA`. The
/// decoded per-vertex normal rides a parallel `[f32; 4]` slice, so the CPU
/// lighting evaluators can read it without changing [`WorldVertex`].
///
/// Returns `None` when the model handle resolves to no MD3 header (a bad handle
/// or a purged model), so the caller skips the surface.
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:1235-1397`
fn decode_md3_surface(
    models: &RenderModels,
    md3_ref: Md3SurfaceRef,
    rgba: [u8; 4],
    sin_table: &[f32; FUNCTABLE_SIZE],
) -> Option<(Vec<WorldVertex>, Vec<u32>, Vec<[f32; 4]>)> {
    let model = models.get_model(md3_ref.h_model);
    let header = *model.md3.get(md3_ref.lod as usize)?;
    if header.is_null() {
        return None;
    }

    // The oracle recomputes backlerp at draw time: a still model (old frame ==
    // current frame) lerps nothing.
    // Source: oracle/codemp/renderer/tr_surface.cpp:1362-1366
    let backlerp = if md3_ref.old_frame == md3_ref.frame {
        0.0
    } else {
        md3_ref.backlerp
    };

    // SAFETY: `header` is the aligned MD3 header block a registered `MOD_MESH`
    // model owns, and `surface_index`/`ofsEnd` walks stay inside that block.
    // `R_AddMD3Surfaces` range-checks `frame`/`old_frame` against
    // `md3[0]->numFrames`, the header count. The stride below assumes every LOD
    // and surface shares that frame count, which the loader upholds for valid
    // MD3 files. A malformed file with a shorter surface reads past the block,
    // the same hole Raven has.
    unsafe {
        let mut surf =
            (header as *const u8).add((*header).ofsSurfaces as usize) as *const md3Surface_t;
        for _ in 0..md3_ref.surface_index {
            surf = (surf as *const u8).add((*surf).ofsEnd as usize) as *const md3Surface_t;
        }

        let (vertices, normals) =
            lerp_md3_vertexes(
                surf,
                md3_ref.frame,
                md3_ref.old_frame,
                backlerp,
                rgba,
                sin_table,
            );

        let num_indexes = ((*surf).numTriangles * 3) as usize;
        let triangles = (surf as *const u8).add((*surf).ofsTriangles as usize) as *const i32;
        let mut indices: Vec<u32> = Vec::with_capacity(num_indexes);
        for j in 0..num_indexes {
            indices.push(*triangles.add(j) as u32);
        }

        Some((vertices, indices, normals))
    }
}

/// Deforms one Ghoul2 (`MOD_MDXM`) surface into GPU vertices and triangle
/// indices. Each vertex sums its weighted bones (`RB_SurfaceGhoul`, the non-gore
/// main arm): the bone matrix per weight comes from the surface's bone-cache
/// `EvalRender`. The decoded per-vertex normal rides a parallel `[f32; 4]`
/// slice, so the CPU lighting evaluators can read it without changing
/// [`WorldVertex`]. The texcoords come from the parallel texcoord array, and
/// the vertex colour is the entity's `shaderRGBA`.
///
/// Returns `None` when the model has no mdxm block or the bone-cache handle is
/// stale, so the caller skips the surface.
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:4060-4451` (the non-gore main
/// arm)
fn decode_ghoul2_surface(
    models: &RenderModels,
    g2: &mut Ghoul2System,
    g2_ref: G2SurfaceRef,
    rgba: [u8; 4],
) -> Option<(Vec<WorldVertex>, Vec<u32>, Vec<[f32; 4]>)> {
    let model = models.get_model(g2_ref.model);
    // The render surface only reaches here for a live `MOD_MDXM` model whose
    // `mdxm` block the loader filled. A null block is not renderable, so it
    // drops rather than reach `mdxm_view_of`'s null deref.
    if model.mdxm.is_null() {
        return None;
    }
    let mdxm = mdxm_view_of(model);
    let surface = mdxm.find_surface(g2_ref.surface_index, g2_ref.lod);

    let num_verts = surface.num_verts();
    let num_tris = surface.num_triangles();

    // The triangle indices are 0-based within this surface's own vertex block.
    let mut indices: Vec<u32> = Vec::with_capacity((num_tris * 3) as usize);
    for j in 0..num_tris {
        let t = surface.triangle(j);
        indices.push(t[0] as u32);
        indices.push(t[1] as u32);
        indices.push(t[2] as u32);
    }

    // A stale or absent bone-cache handle means the skeleton never built, so the
    // surface is not renderable (Raven's null `boneCache`).
    let cache = g2.bone_caches.get_mut(g2_ref.bone_cache)?;

    // This loop is Raven's shipped weight arm (`:4313-4374`): it special-cases
    // 1 and 2 weights (`fBoneWeight * (t1 - t2) + t2` for two weights) and
    // closes the last weight with `1.0f - fTotalWeight` outside the loop. The
    // ghoul2 vertex golden gates the byte-faithful arithmetic.
    // Source: oracle/codemp/renderer/tr_ghoul2.cpp:4313-4374
    let mut vertices: Vec<WorldVertex> = Vec::with_capacity(num_verts as usize);
    let mut normals: Vec<[f32; 4]> = Vec::with_capacity(num_verts as usize);
    for j in 0..num_verts {
        let vert = surface.vert(j);
        let num_weights = vert.num_weights();
        let vert_coords = vert.vert_coords();

        // `bone` is bone 0, the normal bone and the first position bone.
        let bone = cache.eval_render(surface.bone_ref(vert.bone_index(0)));

        // The normal transforms by bone 0 only, a genuine Raven asymmetry: the
        // position blends every weight, the normal does not.
        // Source: oracle/codemp/renderer/tr_ghoul2.cpp:4316-4318
        let vn = vert.normal();
        let normal = [
            bone.matrix[0][0] * vn[0] + bone.matrix[0][1] * vn[1] + bone.matrix[0][2] * vn[2],
            bone.matrix[1][0] * vn[0] + bone.matrix[1][1] * vn[1] + bone.matrix[1][2] * vn[2],
            bone.matrix[2][0] * vn[0] + bone.matrix[2][1] * vn[1] + bone.matrix[2][2] * vn[2],
            0.0,
        ];

        // One row of `DotProduct( m.matrix[row], vertCoords ) + m.matrix[row][3]`.
        let affine = |m: &mdxaBone_t, row: usize| -> f32 {
            m.matrix[row][0] * vert_coords[0]
                + m.matrix[row][1] * vert_coords[1]
                + m.matrix[row][2] * vert_coords[2]
                + m.matrix[row][3]
        };

        let mut p = [0.0f32; 3];
        if num_weights == 1 {
            // One bone, no weight multiply.
            for row in 0..3 {
                p[row] = affine(&bone, row);
            }
        } else {
            let mut bone_weight = vert.bone_weight_not_slow(0);
            if num_weights == 2 {
                // Two bones: interpolate each component between bone 0 and bone 1.
                let bone2 = cache.eval_render(surface.bone_ref(vert.bone_index(1)));
                for row in 0..3 {
                    let t1 = affine(&bone, row);
                    let t2 = affine(&bone2, row);
                    p[row] = bone_weight * (t1 - t2) + t2;
                }
            } else {
                // Three or more bones: weight the first, accumulate the middle
                // weights, then close the last weight with `1.0 - fTotalWeight`.
                for row in 0..3 {
                    p[row] = bone_weight * affine(&bone, row);
                }
                let mut total_weight = bone_weight;
                let mut k = 1;
                while k < num_weights - 1 {
                    let bone = cache.eval_render(surface.bone_ref(vert.bone_index(k)));
                    bone_weight = vert.bone_weight_not_slow(k);
                    total_weight += bone_weight;
                    for row in 0..3 {
                        p[row] += bone_weight * affine(&bone, row);
                    }
                    k += 1;
                }
                let bone = cache.eval_render(surface.bone_ref(vert.bone_index(k)));
                bone_weight = 1.0 - total_weight;
                for row in 0..3 {
                    p[row] += bone_weight * affine(&bone, row);
                }
            }
        }

        let st = surface.texcoord(j);
        vertices.push(WorldVertex {
            position: p,
            st,
            lightmap_st: st,
            color: rgba,
        });
        normals.push(normal);
    }

    Some((vertices, indices, normals))
}

/// The CPU keyframe lerp of one MD3 surface's vertices (`LerpMeshVertexes`),
/// producing one [`WorldVertex`] per vertex and, alongside it, one decoded
/// normal per vertex. Each MD3 vertex is four shorts (xyz plus a packed
/// lat/lng normal), so a frame block is `numVerts * 4` shorts. The `st`
/// texcoords are shared across frames.
///
/// The normal rides a parallel `[f32; 4]` slice (`w` unused), so it never
/// enters the GPU [`WorldVertex`]. The CPU lighting evaluators read it during
/// stage colour evaluation. `sin_table` is `tr.sinTable`, which decodes the
/// packed lat/lng into a unit vector. The `backlerp == 0` frame is already a
/// unit vector, so it needs no normalize. The two-frame blend renormalizes with
/// `VectorArrayNormalize`.
///
/// SAFETY: the caller ([`decode_md3_surface`]) guarantees `surf` is a live,
/// range-checked surface header inside an aligned MD3 block.
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:1235-1346`
unsafe fn lerp_md3_vertexes(
    surf: *const md3Surface_t,
    frame: i32,
    old_frame: i32,
    backlerp: f32,
    rgba: [u8; 4],
    sin_table: &[f32; FUNCTABLE_SIZE],
) -> (Vec<WorldVertex>, Vec<[f32; 4]>) {
    let num_verts = (*surf).numVerts as usize;
    let xyz_base = (surf as *const u8).add((*surf).ofsXyzNormals as usize) as *const i16;
    let st_base = (surf as *const u8).add((*surf).ofsSt as usize) as *const f32;

    // frame block stride = numVerts entries of four shorts each
    let new_base = xyz_base.add(frame as usize * num_verts * 4);
    let old_base = xyz_base.add(old_frame as usize * num_verts * 4);

    let new_scale = MD3_XYZ_SCALE * (1.0 - backlerp);
    let old_scale = MD3_XYZ_SCALE * backlerp;
    let new_normal_scale = 1.0 - backlerp;
    let old_normal_scale = backlerp;

    let mut out: Vec<WorldVertex> = Vec::with_capacity(num_verts);
    let mut normals: Vec<[f32; 4]> = Vec::with_capacity(num_verts);
    for v in 0..num_verts {
        let n = new_base.add(v * 4);
        let (position, normal) = if backlerp == 0.0 {
            let position = [
                *n as f32 * new_scale,
                *n.add(1) as f32 * new_scale,
                *n.add(2) as f32 * new_scale,
            ];
            // The single-frame normal is already a unit vector from the sin
            // table, so the oracle copies it without a normalize.
            let normal = decode_md3_normal(*n.add(3), sin_table);
            (position, [normal[0], normal[1], normal[2], 0.0])
        } else {
            let o = old_base.add(v * 4);
            let position = [
                *o as f32 * old_scale + *n as f32 * new_scale,
                *o.add(1) as f32 * old_scale + *n.add(1) as f32 * new_scale,
                *o.add(2) as f32 * old_scale + *n.add(2) as f32 * new_scale,
            ];
            let new_normal = decode_md3_normal(*n.add(3), sin_table);
            let old_normal = decode_md3_normal(*o.add(3), sin_table);
            let mut normal = [
                old_normal[0] * old_normal_scale + new_normal[0] * new_normal_scale,
                old_normal[1] * old_normal_scale + new_normal[1] * new_normal_scale,
                old_normal[2] * old_normal_scale + new_normal[2] * new_normal_scale,
                0.0,
            ];
            // VectorArrayNormalize renormalizes every blended normal.
            vector_normalize_fast(&mut normal);
            (position, normal)
        };
        let st = [*st_base.add(v * 2), *st_base.add(v * 2 + 1)];
        out.push(WorldVertex {
            position,
            st,
            lightmap_st: st,
            color: rgba,
        });
        normals.push(normal);
    }
    (out, normals)
}

/// `FUNCTABLE_MASK` (`FUNCTABLE_SIZE - 1`) — the sin-table index wrap.
const FUNCTABLE_MASK: usize = FUNCTABLE_SIZE - 1;

/// Decodes one MD3 packed lat/lng normal into a unit vector through `tr.sinTable`.
/// The packed short holds `lat` in the high byte and `lng` in the low byte,
/// each scaled by `FUNCTABLE_SIZE / 256` into a table index.
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:1268-1287`
fn decode_md3_normal(packed: i16, sin_table: &[f32; FUNCTABLE_SIZE]) -> [f32; 3] {
    let packed = packed as i32;
    let lat = ((packed >> 8) & 0xff) * (FUNCTABLE_SIZE as i32 / 256);
    let lng = (packed & 0xff) * (FUNCTABLE_SIZE as i32 / 256);
    // decode X as cos( lat ) * sin( long )
    // decode Y as sin( lat ) * sin( long )
    // decode Z as cos( long )
    [
        sin_table[((lat + (FUNCTABLE_SIZE as i32 / 4)) as usize) & FUNCTABLE_MASK]
            * sin_table[lng as usize],
        sin_table[lat as usize] * sin_table[lng as usize],
        sin_table[((lng + (FUNCTABLE_SIZE as i32 / 4)) as usize) & FUNCTABLE_MASK],
    ]
}

/// `VectorNormalizeFast` — the fast inverse-square-root normalize
/// `VectorArrayNormalize` runs over each blended normal.
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:1180-1227`
fn vector_normalize_fast(v: &mut [f32; 4]) {
    let ilength = Q_rsqrt(v[0] * v[0] + v[1] * v[1] + v[2] * v[2]);
    v[0] *= ilength;
    v[1] *= ilength;
    v[2] *= ilength;
}

fn create_globals_buffer(device: &wgpu::Device, capacity: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("mp_renderer_gpu world globals buffer"),
        size: capacity as u64 * GLOBALS_STRIDE,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn create_globals_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("mp_renderer_gpu world globals bind group"),
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer,
                offset: 0,
                size: wgpu::BufferSize::new(CLIP_MATRIX_SIZE),
            }),
        }],
    })
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
    use mp_renderer::tr_curve::empty_grid_mesh;
    use mp_renderer::tr_local::acff_t::acff_t;
    use mp_renderer::tr_local::eglfog_override::EGLFogOverride;
    use mp_renderer::tr_local::gen_func_t::genFunc_t;
    use mp_renderer::tr_local::tex_mod_info_t::texModInfo_t;
    use mp_renderer::tr_local::wave_form_t::waveForm_t;

    use crate::ui_host::boot::empty_assets;

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

    #[test]
    fn grid_lod_indices_with_zero_stored_error_equal_the_static_set() {
        // A zero stored error keeps every interior row and column, so the
        // reduced set must equal the static keep-all set the goldens use.
        let mut grid = empty_grid_mesh();
        grid.width = 3;
        grid.height = 2;
        grid.width_lod_error = vec![0.0; 3];
        grid.height_lod_error = vec![0.0; 2];
        let ori = orientationr_t {
            origin: [0.0; 3],
            axis: [[0.0; 3]; 3],
            viewOrigin: [0.0; 3],
            modelMatrix: [0.0; 16],
        };
        let view = zeroed_view();

        let lod = grid_lod_indices(&grid, &ori, &view, 250.0);
        assert_eq!(lod, grid_indices(3, 2));
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

    // stage classification and dynamic-block evaluation

    use mp_renderer::tr_local::tex_mod_t::texMod_t;

    fn flat_wave() -> waveForm_t {
        waveForm_t {
            func: genFunc_t::GF_NONE,
            base: 0.0,
            amplitude: 0.0,
            phase: 0.0,
            frequency: 0.0,
        }
    }

    fn empty_bundle() -> TextureBundle {
        TextureBundle {
            image: None,
            tc_gen: texCoordGen_t::TCGEN_TEXTURE,
            tc_gen_vectors: [[0.0; 3]; 2],
            tex_mods: Vec::new(),
            num_image_animations: 0,
            image_animation_speed: 0.0,
            is_lightmap: false,
            one_shot_anim_map: false,
            vertex_lightmap: false,
            is_video_map: false,
            video_map_handle: 0,
            image_animations: Vec::new(),
        }
    }

    fn plain_stage() -> ShaderStage {
        ShaderStage {
            active: true,
            is_detail: false,
            index: 0,
            lightmap_style: 0,
            bundle: std::array::from_fn(|_| empty_bundle()),
            rgb_wave: flat_wave(),
            rgb_gen: colorGen_t::CGEN_IDENTITY,
            alpha_wave: flat_wave(),
            alpha_gen: alphaGen_t::AGEN_IDENTITY,
            constant_color: [255, 255, 255, 255],
            state_bits: 0,
            adjust_colors_for_fog: acff_t::ACFF_NONE,
            gl_fog_color_override: EGLFogOverride::GLFOGOVERRIDE_NONE,
            ss: None,
            glow: false,
        }
    }

    fn scroll_tex_mod(translate: [f32; 2]) -> texModInfo_t {
        texModInfo_t {
            r#type: texMod_t::TMOD_SCROLL,
            wave: flat_wave(),
            matrix: [[0.0; 2]; 2],
            translate,
        }
    }

    fn cpu_vertex(st: [f32; 2], lightmap_st: [f32; 2], color: [u8; 4]) -> WorldVertex {
        WorldVertex {
            position: [0.0, 0.0, 0.0],
            st,
            lightmap_st,
            color,
        }
    }

    // static-vs-dynamic classification

    #[test]
    fn an_identity_colour_stage_is_dynamic() {
        // `CGEN_IDENTITY` writes white, which the static BSP vertex colour does
        // not hold, so the stage needs `ComputeColors` on a dynamic block.
        assert!(stage_is_dynamic(&plain_stage(), false));
    }

    #[test]
    fn a_vertex_passthrough_stage_is_static() {
        // `CGEN_EXACT_VERTEX` with `AGEN_SKIP` keeps the BSP vertex colour, so
        // the static world buffer serves it.
        let mut stage = plain_stage();
        stage.rgb_gen = colorGen_t::CGEN_EXACT_VERTEX;
        stage.alpha_gen = alphaGen_t::AGEN_SKIP;
        assert!(!stage_is_dynamic(&stage, false));
    }

    #[test]
    fn a_tcmod_stage_is_dynamic() {
        let mut stage = plain_stage();
        stage.bundle[0].tex_mods = vec![scroll_tex_mod([0.0, 0.025])];
        assert!(stage_is_dynamic(&stage, false));
        // A two-texture collapse still needs the dynamic block for tcMods.
        assert!(stage_is_dynamic(&stage, true));
    }

    #[test]
    fn a_waveform_colour_stage_is_dynamic_only_as_single_texture() {
        let mut stage = plain_stage();
        stage.rgb_gen = colorGen_t::CGEN_WAVEFORM;
        assert!(stage_is_dynamic(&stage, false));
        // A two-texture collapse ignores the vertex colour, so a waveform gen
        // alone keeps it static.
        assert!(!stage_is_dynamic(&stage, true));

        let mut alpha_stage = plain_stage();
        alpha_stage.alpha_gen = alphaGen_t::AGEN_WAVEFORM;
        assert!(stage_is_dynamic(&alpha_stage, false));
    }

    // multitexture-env collapse decision

    #[test]
    fn modulate_env_with_a_lightmap_second_bundle_collapses() {
        let mut stage = plain_stage();
        stage.bundle[1].image = Some(ImageHandle::new(7, 0));
        stage.bundle[1].is_lightmap = true;
        let shader = ShaderAsset {
            multitexture_env: GL_MODULATE,
            ..Default::default()
        };
        assert!(is_modulate_collapse(&shader, &stage));
    }

    #[test]
    fn modulate_env_with_a_non_lightmap_second_bundle_does_not_collapse() {
        // The collapse feeds lightmap st to texture unit 1, so a non-lightmap
        // bundle 1 has no collapse path and draws bundle 0 alone.
        let mut stage = plain_stage();
        stage.bundle[1].image = Some(ImageHandle::new(7, 0));
        let shader = ShaderAsset {
            multitexture_env: GL_MODULATE,
            ..Default::default()
        };
        assert!(!is_modulate_collapse(&shader, &stage));
    }

    #[test]
    fn a_non_modulate_env_does_not_collapse() {
        let mut stage = plain_stage();
        stage.bundle[1].image = Some(ImageHandle::new(7, 0));
        // `GL_ADD` = 0x0104 is a real env, but only `GL_MODULATE` keeps the
        // two-texture pass here.
        let shader = ShaderAsset {
            multitexture_env: 0x0104,
            ..Default::default()
        };
        assert!(!is_modulate_collapse(&shader, &stage));
    }

    #[test]
    fn modulate_env_without_a_second_image_does_not_collapse() {
        let shader = ShaderAsset {
            multitexture_env: GL_MODULATE,
            ..Default::default()
        };
        assert!(!is_modulate_collapse(&shader, &plain_stage()));
    }

    // per-vertex texcoord evaluation

    #[test]
    fn a_scroll_tcmod_offsets_every_vertex_at_a_fixed_time() {
        // `tcMod scroll 0 0.025` at t = 4s adds 0.1 in t to each vertex, nothing
        // in s, from each vertex's own base st.
        let mut stage = plain_stage();
        stage.bundle[0].tex_mods = vec![scroll_tex_mod([0.0, 0.025])];

        let cpu = [
            cpu_vertex([0.0, 0.0], [0.5, 0.5], [10, 20, 30, 40]),
            cpu_vertex([1.0, 0.25], [0.6, 0.6], [10, 20, 30, 40]),
            cpu_vertex([0.5, 0.75], [0.7, 0.7], [10, 20, 30, 40]),
        ];

        let assets = empty_assets();
        let noise = NoiseState::default();
        let mut warnings = Stage2dWarnings::default();
        let mut out: Vec<WorldVertex> = Vec::new();
        let base = build_dynamic_block(
            &cpu,
            &stage,
            StSource::Base,
            false,
            StageTime::new(4.0, 0.0),
            &noise,
            &assets,
            "test",
            None,
            None,
            None,
            &mut warnings,
            &mut out,
            EntityFx::default(),
            0,
            [1.0, 0.0, 0.0],
        );

        assert_eq!(base, 0);
        assert_eq!(out.len(), cpu.len());
        for (moved, original) in out.iter().zip(cpu) {
            assert!((moved.st[0] - original.st[0]).abs() < 1e-6);
            assert!(
                (moved.st[1] - (original.st[1] + 0.1)).abs() < 1e-6,
                "{:?} vs {:?}",
                moved.st,
                original.st
            );
            // The position and lightmap st copy through unchanged.
            assert_eq!(moved.lightmap_st, original.lightmap_st);
        }
    }

    #[test]
    fn a_lightmap_source_seeds_the_dynamic_block_from_lightmap_st() {
        // A `TCGEN_LIGHTMAP` stage with a tcMod reads the lightmap st, so the
        // dynamic block starts from `lightmap_st`, not `st`.
        let mut stage = plain_stage();
        stage.bundle[0].tc_gen = texCoordGen_t::TCGEN_LIGHTMAP;
        stage.bundle[0].tex_mods = vec![scroll_tex_mod([0.0, 0.0])];

        let cpu = [cpu_vertex([0.1, 0.2], [0.8, 0.9], [1, 2, 3, 4])];

        let assets = empty_assets();
        let noise = NoiseState::default();
        let mut warnings = Stage2dWarnings::default();
        let mut out: Vec<WorldVertex> = Vec::new();
        build_dynamic_block(
            &cpu,
            &stage,
            StSource::Lightmap,
            false,
            StageTime::new(0.0, 0.0),
            &noise,
            &assets,
            "test",
            None,
            None,
            None,
            &mut warnings,
            &mut out,
            EntityFx::default(),
            0,
            [1.0, 0.0, 0.0],
        );

        assert_eq!(out[0].st, [0.8, 0.9]);
    }

    // entity clip-matrix slot assignment

    /// `QSORT_ENTITYNUM_SHIFT` (`tr_main`), restated for the test's sort keys.
    const QSORT_ENTITYNUM_SHIFT: u32 = 7;

    fn assets_with_one_shader() -> RenderAssets {
        let mut assets = empty_assets();
        // `R_DecomposeSort` indexes `sorted_shaders`, so it needs one entry.
        let handle = assets
            .shaders
            .handle_at_slot(0)
            .expect("slot 0 exists in a fresh shader arena");
        assets.sorted_shaders.push(handle);
        assets
    }

    fn zeroed_view() -> viewParms_t {
        // SAFETY: `viewParms_t` is a frozen `#[repr(C)]` POD.
        unsafe { core::mem::zeroed() }
    }

    fn zeroed_entity() -> trRefEntity_t {
        // SAFETY: `trRefEntity_t` is a frozen `#[repr(C)]` POD. A zeroed value
        // has `reType == RT_MODEL`, the arm `R_RotateForEntity` rotates.
        unsafe { core::mem::zeroed() }
    }

    #[test]
    fn a_world_only_draw_list_uses_slot_zero() {
        // The world entity always maps to slot 0, so an entity-free draw list
        // builds exactly one clip-matrix slot.
        let assets = assets_with_one_shader();
        let view = zeroed_view();
        let entities: Vec<trRefEntity_t> = Vec::new();
        let mut scratch = TrMainScratch {
            pre_trans_ent_matrix: [0.0; 16],
        };
        let draw_surfs = vec![DrawSurf {
            sort: (TR_WORLDENT as u32) << QSORT_ENTITYNUM_SHIFT,
            surface: SurfaceGeometry::Other,
        }];

        let (clips, _oris, slot_map) =
            build_entity_slots(&draw_surfs, &assets, &view, &entities, &mut scratch);

        assert_eq!(clips.len(), 1);
        assert_eq!(slot_map.get(&TR_WORLDENT), Some(&0));
    }

    #[test]
    fn each_distinct_entity_gets_its_own_slot_with_world_at_zero() {
        // The world is slot 0, and each real entity number gets the next slot
        // in first-appearance order.
        let assets = assets_with_one_shader();
        let view = zeroed_view();
        let entities = vec![zeroed_entity(), zeroed_entity()];
        let mut scratch = TrMainScratch {
            pre_trans_ent_matrix: [0.0; 16],
        };
        let draw_surfs = vec![
            DrawSurf {
                sort: 1u32 << QSORT_ENTITYNUM_SHIFT,
                surface: SurfaceGeometry::Other,
            },
            DrawSurf {
                sort: (TR_WORLDENT as u32) << QSORT_ENTITYNUM_SHIFT,
                surface: SurfaceGeometry::Other,
            },
            DrawSurf {
                sort: 0u32 << QSORT_ENTITYNUM_SHIFT,
                surface: SurfaceGeometry::Other,
            },
            // A repeat of entity 1 must not add a second slot.
            DrawSurf {
                sort: 1u32 << QSORT_ENTITYNUM_SHIFT,
                surface: SurfaceGeometry::Other,
            },
        ];

        let (clips, _oris, slot_map) =
            build_entity_slots(&draw_surfs, &assets, &view, &entities, &mut scratch);

        // World plus entity 1 plus entity 0 is three distinct slots.
        assert_eq!(clips.len(), 3);
        assert_eq!(slot_map.get(&TR_WORLDENT), Some(&0));
        assert_eq!(slot_map.get(&1), Some(&1));
        assert_eq!(slot_map.get(&0), Some(&2));
    }

    // md3 keyframe decode

    /// Builds an in-memory MD3 surface: one vertex, two frames, packed into the
    /// on-disk layout `lerp_md3_vertexes` walks. Frame 0 is (100, 200, 300),
    /// frame 1 is (200, 400, 600) in raw shorts; the st is (0.25, 0.5).
    fn build_two_frame_surface() -> Vec<u8> {
        let header_size = size_of::<md3Surface_t>(); // 108
        let ofs_xyz = header_size as i32; // 108
        let ofs_st = ofs_xyz + 2 * 1 * 4 * 2; // 2 frames * 1 vert * 4 shorts * 2 bytes = 124
        let ofs_end = ofs_st + 2 * 4; // one vert of two f32 st = 132

        let mut buf = vec![0u8; ofs_end as usize];

        let surf = md3Surface_t {
            ident: 0,
            name: [0; 64],
            flags: 0,
            numFrames: 2,
            numShaders: 0,
            numVerts: 1,
            numTriangles: 0,
            ofsTriangles: ofs_end,
            ofsShaders: header_size as i32,
            ofsSt: ofs_st,
            ofsXyzNormals: ofs_xyz,
            ofsEnd: ofs_end,
        };
        // SAFETY: `md3Surface_t` is a `#[repr(C)]` POD; its bytes copy into the
        // buffer head, matching the on-disk layout.
        unsafe {
            std::ptr::copy_nonoverlapping(
                &surf as *const md3Surface_t as *const u8,
                buf.as_mut_ptr(),
                header_size,
            );
        }

        // Frame 0 vertex, then frame 1 vertex, four shorts each (xyz + normal).
        let xyz: [i16; 8] = [100, 200, 300, 0, 200, 400, 600, 0];
        let st: [f32; 2] = [0.25, 0.5];
        // SAFETY: the writes land inside the buffer at the offsets above.
        unsafe {
            std::ptr::copy_nonoverlapping(
                xyz.as_ptr() as *const u8,
                buf.as_mut_ptr().add(ofs_xyz as usize),
                xyz.len() * size_of::<i16>(),
            );
            std::ptr::copy_nonoverlapping(
                st.as_ptr() as *const u8,
                buf.as_mut_ptr().add(ofs_st as usize),
                st.len() * size_of::<f32>(),
            );
        }
        buf
    }

    #[test]
    fn lerp_md3_at_backlerp_zero_takes_the_current_frame() {
        let buf = build_two_frame_surface();
        let sin_table = [0.0f32; FUNCTABLE_SIZE];
        // SAFETY: the buffer head is a valid `md3Surface_t` from the builder.
        let (verts, _normals) = unsafe {
            lerp_md3_vertexes(
                buf.as_ptr() as *const md3Surface_t,
                1,
                0,
                0.0,
                [10, 20, 30, 40],
                &sin_table,
            )
        };
        assert_eq!(verts.len(), 1);
        // frame 1 (200, 400, 600) times MD3_XYZ_SCALE (1/64)
        assert_eq!(
            verts[0].position,
            [200.0 / 64.0, 400.0 / 64.0, 600.0 / 64.0]
        );
        assert_eq!(verts[0].st, [0.25, 0.5]);
        assert_eq!(verts[0].color, [10, 20, 30, 40]);
    }

    #[test]
    fn lerp_md3_at_backlerp_one_takes_the_old_frame() {
        let buf = build_two_frame_surface();
        let sin_table = [0.0f32; FUNCTABLE_SIZE];
        // SAFETY: see above.
        let (verts, _normals) = unsafe {
            lerp_md3_vertexes(
                buf.as_ptr() as *const md3Surface_t,
                1,
                0,
                1.0,
                [255, 255, 255, 255],
                &sin_table,
            )
        };
        // frame 0 (100, 200, 300) times MD3_XYZ_SCALE
        assert_eq!(
            verts[0].position,
            [100.0 / 64.0, 200.0 / 64.0, 300.0 / 64.0]
        );
    }

    #[test]
    fn lerp_md3_at_backlerp_half_blends_the_two_frames() {
        let buf = build_two_frame_surface();
        let sin_table = [0.0f32; FUNCTABLE_SIZE];
        // SAFETY: see above.
        let (verts, _normals) = unsafe {
            lerp_md3_vertexes(
                buf.as_ptr() as *const md3Surface_t,
                1,
                0,
                0.5,
                [255, 255, 255, 255],
                &sin_table,
            )
        };
        // half of each frame: (100+200)/2, (200+400)/2, (300+600)/2, all /64
        assert_eq!(
            verts[0].position,
            [150.0 / 64.0, 300.0 / 64.0, 450.0 / 64.0]
        );
    }
}
