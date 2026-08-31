//! Raven `tr_bsp.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_bsp.cpp`

// Raven-named functions/types keep their original casing across this
// transcription, matching the rest of the renderer/engine crates.
#![allow(non_snake_case)]
// Wave-0 ports of Raven `static` helpers: private by fidelity, with their
// callers landing in later R3 waves.
#![allow(dead_code)]

use core::mem::{replace, size_of};
use std::sync::Arc;

use mp_engine_qcommon::cm_load::CM_TakeCachedMapDiskImage;
use mp_engine_qcommon::common::{com_error, com_printf, Common, EngineHostView};
use mp_engine_qcommon::files_common::FS_ReadFileVec;
use mp_engine_qcommon::qfiles::bsp_limits::BSP_VERSION;
use mp_engine_qcommon::qfiles::dbrush_t::dbrush_t;
use mp_engine_qcommon::qfiles::dbrushside_t::dbrushside_t;
use mp_engine_qcommon::qfiles::dfog_t::dfog_t;
use mp_engine_qcommon::qfiles::dheader_t::HEADER_LUMPS;
use mp_engine_qcommon::qfiles::dleaf_t::dleaf_t;
use mp_engine_qcommon::qfiles::dmodel_t::dmodel_t;
use mp_engine_qcommon::qfiles::dnode_t::dnode_t;
use mp_engine_qcommon::qfiles::dplane_t::dplane_t;
use mp_engine_qcommon::qfiles::draw_vert_t::{drawVert_t, MAXLIGHTMAPS};
use mp_engine_qcommon::qfiles::dshader_t::dshader_t;
use mp_engine_qcommon::qfiles::dsurface_t::dsurface_t;
use mp_engine_qcommon::qfiles::lump_indices::{
    LUMP_BRUSHES, LUMP_BRUSHSIDES, LUMP_DRAWINDEXES, LUMP_DRAWVERTS, LUMP_ENTITIES, LUMP_FOGS,
    LUMP_LEAFS, LUMP_LEAFSURFACES, LUMP_LIGHTARRAY, LUMP_LIGHTGRID, LUMP_LIGHTMAPS, LUMP_MODELS,
    LUMP_NODES, LUMP_PLANES, LUMP_SHADERS, LUMP_SURFACES, LUMP_VISIBILITY,
};
use mp_engine_qcommon::qfiles::lump_t::lump_t;
use mp_engine_qcommon::qfiles::map_surface_type_t::mapSurfaceType_t;
use mp_engine_qcommon::qfiles::map_vert_t::mapVert_t;
use mp_engine_qcommon::qfiles::shader_limits::{SHADER_MAX_INDEXES, SHADER_MAX_VERTEXES};
use mp_qshared::shared::com_parse::QSharedScratch;
use mp_qshared::shared::q_math::PlaneTypeForNormal;
use mp_qshared::shared::q_string::{COM_Parse, COM_StripExtension};
use mp_qshared::shared::swap::{LittleFloat, LittleLong};
use mp_qshared::shared::{
    cplane_t, errorParm_t, qfalse, qtrue, VectorNormalize, MAX_QPATH, SURF_NODRAW,
};
use native_math::qmath::{
    _DotProduct, _VectorAdd, _VectorScale, _VectorSubtract, vec3_origin, AddPointToBounds,
    ClearBoundsMP, ColorBytes4, VectorLength,
};
use native_string::sscanf::sscanf_f32s;

use crate::gl_constants::{GL_CLAMP, GL_RGBA};
use crate::render_state::world_load_state::WorldLoadState;
use crate::tr_scene::SceneState;
use crate::render_state::image_asset::ImageHandle;
use crate::render_state::placeholders::{Vec3, WorldAsset};
use crate::render_state::render_assets::RenderAssets;
use crate::render_state::renderer_cvars::RendererCvars;
use crate::render_state::shader_asset::ShaderHandle;
use crate::tr_arioche::R_RMGInit;
use crate::tr_cmds::R_SyncRenderThread;
use crate::tr_curve::{
    empty_grid_mesh, GridMesh, R_GridInsertColumn, R_GridInsertRow, R_SubdividePatchToGrid,
    MAX_GRID_SIZE,
};
use crate::tr_image::{R_CreateImage, TrImageState};
use crate::tr_local::fog_parms_t::fogParms_t;
use crate::tr_local::fog_t::fog_t;
use crate::tr_local::mgrid_t::mgrid_t;
use crate::tr_local::srf_flare_s::srfFlare_t;
use crate::tr_local::surface_type_t::surfaceType_t;
use crate::tr_local::view_parms_t::viewParms_t;
use crate::tr_model::render_models::RenderModels;
use crate::tr_shader::{
    lightmapsVertex, stylesDefault, FogParms, RE_RegisterShader, R_FindShader, R_RemapShader,
    LIGHTMAP_BY_VERTEX, LIGHTMAP_NONE,
};
use crate::tr_worldeffects::world_effects::WorldEffectsState;

// This wave threads `WorldAsset` (`crate::render_state::placeholders`) and
// `FrameState` (`crate::render_state::frame_state`) as the fns below expect
// them, per the R2 `## State ownership` rows for `tr.world`/`s_worldData`
// and `tr` frontend scratch. The fields these fns read/write —
// `WorldAsset`: `name`, `shaders: Vec<DShader>`, `mark_surfaces: Vec<u32>`,
// `planes: Vec<cplane_t>`, `light_grid_bounds: [i32; 3]`,
// `num_grid_array_elements: i32`, `light_grid_array: Vec<u16>`,
// `light_grid_data: Option<Vec<mgrid_t>>`, `num_clusters`/`cluster_bytes:
// i32`, `vis`/`novis: Vec<u8>`, `entity_string: String`,
// `entity_parse_point: usize`; `FrameState`: `overbright_bits: i32`,
// `external_vis_data: Option<Vec<u8>>` — are the tier-2 transition audit's
// licensed shapes (Group 1) and landed with this wave's field-merge step.
// (`light_grid_data` merged to `tr_light.rs`'s typed `Vec<mgrid_t>` shape,
// not this note's original `Vec<u8>` guess — this file only ever writes
// `None` through it.)
//
// WAVE 1 ADDITIONS, landed by the same field-merge step: `WorldAsset::nodes:
// Vec<Node>` / `num_decision_nodes: i32` (`R_LoadNodesAndLeafs`, tier-2
// transition audit Group 1 `mnode_t`/`world_t` rows), `WorldAsset::bmodels:
// Vec<BModel>` (`R_LoadLightGrid` reads `bmodels[0].bounds`; populated by
// the not-yet-ported `R_LoadSubmodels`, out of this wave's packet), and
// `WorldAsset::light_grid_size: Vec3` (`R_LoadLightGrid` reads it; populated
// by the not-yet-ported `R_LoadEntities`'s `gridsize` worldspawn key,
// `oracle/codemp/renderer/tr_bsp.cpp:1887-1889,1956`, out of this wave's
// packet). `R_FixSharedVertexLodError_r`/`R_MovePatchSurfacesToHunk`
// deliberately do NOT thread `WorldAsset` — the packet's threading digest
// marks both "no state channel"/"engine seam" only; they take
// `worldData.surfaces` as a plain `&mut [Surface]` parameter instead (see
// each fn's doc comment).
//
// WAVE 8 ADDITIONS, landed by the same field-merge step: `WorldAsset::fogs:
// Vec<Fog>` and `WorldAsset::global_fog: i32` (`R_LoadFogs`, tier-2
// transition audit Group 1 `fog_t`/`world_t` rows — `fog_t`'s owning wave).
// `worldData.numfogs` collapses to `fogs.len()` (no separate count field, same
// collection-length-is-the-count pattern as this file's other `Vec`-backed
// counts). `Fog` (this file) is the owned `fog_t` replacement.

// WAVE 10 ADDITIONS, landed by the same field-merge step: `FrameState::
// sun_ambient: Vec3` (`R_LoadEntities`, R2 `## State ownership`'s `tr`
// frontend-scratch row — "sun/fog fields" — `FrameState` carries
// `sun_direction` already but not `sunAmbient`).
//
// DEC-43 ADDITIONS: `WorldAsset::surfaces: Vec<Surface>` — the owned
// `worldData.surfaces` carrier (`Surface`/`SurfaceData`, defined beside the
// `Parse*` payload types below). `R_LoadSurfaces` fills it in lump order and
// the five patch-stitching walks
// (`R_FixSharedVertexLodError`/`_r`/`R_StitchPatches`/`R_TryStitchingPatch`/
// `R_StitchAllPatches`) iterate it over the oracle's own `0..numsurfaces`
// index domain, so their transcribed `surfaceType != SF_GRID` guards are
// live `SurfaceData::Grid` matches rather than always-true checks over a
// grids-only stand-in collection.

// The two dependencies `R_LoadFogs`'s shader/fog-parameter step needs are
// both landed now (waves-7-13 fix round): `stylesDefault` (the `const byte
// stylesDefault[MAXLIGHTMAPS]` table, `tr_shader.rs`) and
// `ShaderAsset::fog_parms` (persisted by `GeneratePermanentShader` instead of
// being dropped by `FinishShader`). That step is transcribed in full below.

// WAVE 11 ADDITIONS, landed by the same field-merge step: `RenderAssets::
// world_map_loaded: bool` (`RE_LoadWorldMap_Actual`, `_PREAMBLE.md`'s
// `trGlobals_t` tier-2 transition-audit row — `registered`/`worldMapLoaded`
// share the same "session flag, not per-frame scratch" disposition) and
// `WorldAsset::base_name: String` (the `world_t` row's second of its
// "`String` x2 for the names" pair — `name`/`baseName`).

/// `fileBase` — the file-scope byte pointer the oracle's BSP loaders index
/// by lump offset (`fileBase + l->fileofs`). Not part of R2's frozen state
/// vocabulary (no row in `## State ownership`); named here per DEC-37 A13.3.
/// Genuinely scoped to one synchronous BSP-file load, never held across
/// frames — owned by whichever caller drives `R_LoadWorld`'s call tree and
/// threaded through by `&`.
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp` (`fileBase`, file-scope static)
pub struct BspLoadContext {
    pub file_base: Vec<u8>,
}

/// Owned replacement for Raven `dshader_t`'s on-disk shader reference —
/// `world_t`'s `Vec<DShader>` shape (tier-2 transition audit, Group 1:
/// `world_t` row).
///
/// Type definition source: `oracle/codemp/qcommon/qfiles.h:447-451`
#[derive(Clone)]
pub struct DShader {
    pub shader: String,
    pub surface_flags: i32,
    pub content_flags: i32,
}

/// Owned replacement for Raven `mnode_t`'s parent/children pointer graph —
/// index-linked into a shared node arena instead of raw pointers (tier-2
/// transition audit, Group 1: `mnode_t` row). Wave 0 only needed the fields
/// `R_SetParent` touches; this wave (`R_LoadNodesAndLeafs`, the node/leaf
/// loader) adds the remaining fields it fills — `plane` becomes an index
/// into `WorldAsset::planes` and `firstmarksurface` an index into
/// `WorldAsset::mark_surfaces`, per the licensed replacement shape.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:917-934`
#[derive(Clone)]
pub struct Node {
    pub parent: Option<usize>,
    pub children: [Option<usize>; 2],
    /// `CONTENTS_NODE` (`-1`, `oracle/codemp/renderer/tr_local.h:882`) for
    /// decision nodes, to differentiate from leafs. Raven's
    /// `R_LoadNodesAndLeafs` never assigns `contents` for leaf entries (only
    /// `mins`/`maxs`/`cluster`/`area`/`firstmarksurface`/`nummarksurfaces`
    /// are set in that loop) — leafs keep the zero-initialized value here,
    /// same as the oracle's zeroed `Hunk_Alloc` block.
    pub contents: i32,
    // W2-F4 moved `visframe` out of the node. The loaded world is immutable
    // after load, so `R_MarkLeaves` stamps `WorldWalkScratch::node_visframe`
    // at this node's index instead.
    // Source: `oracle/codemp/renderer/tr_local.h:919`
    /// `mins`/`maxs` — frustum-culling bounds (decision nodes and leafs
    /// alike).
    pub mins: [i32; 3],
    pub maxs: [i32; 3],
    /// `plane` — index into `WorldAsset::planes` (decision nodes only; not
    /// meaningful for leafs).
    pub plane: Option<usize>,
    /// `cluster` (leafs only).
    pub cluster: i32,
    /// `area` (leafs only).
    pub area: i32,
    /// `firstmarksurface` — start index into `WorldAsset::mark_surfaces`
    /// (leafs only).
    pub firstmarksurface: usize,
    /// `nummarksurfaces` (leafs only).
    pub nummarksurfaces: i32,
}

/// Owned replacement for Raven `bmodel_t`'s culling bounds — wave 1
/// (`R_LoadLightGrid`) only reads `bounds`; this wave (`R_LoadSubmodels`)
/// adds `first_surface`/`num_surfaces` (tier-2 transition audit, Group 1:
/// `bmodel_t` row). `first_surface` is an index, not a pointer — same
/// treatment as `R_LoadMarksurfaces`'s `mark_surfaces` above.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:938-942`
#[derive(Clone)]
pub struct BModel {
    pub bounds: [Vec3; 2],
    /// `firstSurface` — start of this submodel's `[first, first + num)` range
    /// into [`WorldAsset::surfaces`] (DEC-43.1).
    pub first_surface: usize,
    /// `numSurfaces`.
    pub num_surfaces: i32,
}

/// Owned replacement for Raven `fog_t` — one fog volume, `WorldAsset::fogs`'
/// element (tier-2 transition audit, Group 1: `fog_t` row — `hasSurface:
/// qboolean` → `bool`; wave 8, `R_LoadFogs`, is `fog_t`'s owning wave).
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:616-627`
#[derive(Clone, Copy, Default)]
pub struct Fog {
    /// `originalBrushNumber` — `-1` for a global/"no brush" fog (the
    /// `MIN_WORLD_COORD`/`MAX_WORLD_COORD` bounds sentinel below).
    pub original_brush_number: i32,
    /// `bounds[2]`.
    pub bounds: [Vec3; 2],
    /// `colorInt` — packed byte format.
    pub color_int: u32,
    /// `tcScale` — texture coordinate vector scale.
    pub tc_scale: f32,
    /// `parms` (`fogParms_t`).
    pub parms: FogParms,
    /// `hasSurface` — for clipping distance in fog when outside.
    pub has_surface: bool,
    /// `surface` — the gradient vector/plane (`[normal.x, normal.y,
    /// normal.z, -dist]`).
    pub surface: [f32; 4],
}

impl Fog {
    /// The ABI `fog_t` copy of this fog volume, for the render pass.
    ///
    /// `R_RenderView`, `RB_FogPass`, and the `RB_CalcModulate*ByFog` family read
    /// `fog_t`, so the render side copies each `Fog` into one before the frame.
    /// The `Fog` list is 1-indexed (slot 0 is a dummy), so a per-element copy
    /// keeps `fogNum` addressing intact.
    ///
    /// Type definition source: `oracle/codemp/renderer/tr_local.h:616-627`
    pub fn to_fog_t(&self) -> fog_t {
        fog_t {
            originalBrushNumber: self.original_brush_number,
            bounds: self.bounds,
            colorInt: self.color_int,
            tcScale: self.tc_scale,
            parms: fogParms_t {
                color: self.parms.color,
                depthForOpaque: self.parms.depth_for_opaque,
            },
            hasSurface: if self.has_surface { qtrue } else { qfalse },
            surface: self.surface,
        }
    }
}

/// Raven `MAX_FACE_POINTS`.
/// Source: `oracle/codemp/renderer/tr_local.h:685`
const MAX_FACE_POINTS: i32 = 64;

/// Raven `MAX_WORLD_COORD`.
/// Source: `oracle/codemp/game/q_shared.h:18`
const MAX_WORLD_COORD: f32 = 64.0 * 1024.0;

/// Raven `MIN_WORLD_COORD`.
/// Source: `oracle/codemp/game/q_shared.h:19`
const MIN_WORLD_COORD: f32 = -64.0 * 1024.0;

/// Decodes a Latin-1, NUL-terminated fixed-size name buffer (the on-disk BSP
/// lump convention) into an owned `String`. Inlined rather than importing
/// `native_string::latin1_to_string` — that crate is not a dependency of
/// `mp_renderer`; Latin-1 discipline still holds (each byte maps 1:1 to its
/// Unicode codepoint, `from_utf8_lossy` is never used).
fn latin1_name(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    bytes[..end].iter().map(|&b| b as char).collect()
}

/// Raven `HSVtoRGB`.
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp:27-75`
fn HSVtoRGB(h: f32, s: f32, v: f32) -> [f32; 3] {
    let h = h * 5.0;

    let i = h.floor() as i32;
    let f = h - i as f32;

    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));

    match i {
        0 => [v, t, p],
        1 => [q, v, p],
        2 => [p, v, t],
        3 => [p, q, v],
        4 => [t, p, v],
        // PORT-NOTE: the oracle `switch` has no default arm (`rgb` is left
        // uninitialized for any `i` outside 0..=5, UB) — `i` is always in
        // 0..=5 for `h` in `[0, 1)` after the `h *= 5` scale (only `h ==
        // 1.0` reaches `i == 5`), so this arm covers case 5 and picks the
        // one defined behavior for the otherwise-unreachable remainder
        // (porting-rules §19).
        _ => [v, p, q],
    }
}

/// Raven `R_ColorShiftLightingBytes` (4-component, `in`→`out`). //rwwRMG - modified
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp:83-119`
pub fn R_ColorShiftLightingBytes(world_load: &WorldLoadState, color_in: [u8; 4]) -> [u8; 4] {
    // should NOT do it if overbrightBits is 0
    let mut shift = 0i32;
    if world_load.overbright_bits != 0 {
        shift = 1 - world_load.overbright_bits;
    }

    if shift == 0 {
        return color_in;
    }

    // shift the data based on overbright range
    let mut r = (color_in[0] as i32) << shift;
    let mut g = (color_in[1] as i32) << shift;
    let mut b = (color_in[2] as i32) << shift;

    // normalize by color instead of saturating to white
    if (r | g | b) > 255 {
        let mut max = if r > g { r } else { g };
        max = if max > b { max } else { b };
        r = r * 255 / max;
        g = g * 255 / max;
        b = b * 255 / max;
    }

    [r as u8, g as u8, b as u8, color_in[3]]
}

/// Raven `R_ColorShiftLightingBytes` (3-component, in-place).
///
/// PORT-NOTE: Raven overloads this name with a 3-byte in-place variant
/// (`tr_bsp.cpp:127-159`); Rust has no overloading, so this symbol is
/// disambiguated as `R_ColorShiftLightingBytesRGB`, returning the shifted
/// bytes instead of mutating `in` in place.
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp:127-159`
fn R_ColorShiftLightingBytesRGB(world_load: &WorldLoadState, color_in: [u8; 3]) -> [u8; 3] {
    let mut shift = 0i32;
    if world_load.overbright_bits != 0 {
        shift = 1 - world_load.overbright_bits;
    }

    if shift == 0 {
        return color_in;
    }

    let mut r = (color_in[0] as i32) << shift;
    let mut g = (color_in[1] as i32) << shift;
    let mut b = (color_in[2] as i32) << shift;

    if (r | g | b) > 255 {
        let mut max = if r > g { r } else { g };
        max = if max > b { max } else { b };
        r = r * 255 / max;
        g = g * 255 / max;
        b = b * 255 / max;
    }

    [r as u8, g as u8, b as u8]
}

/// Raven `RE_SetWorldVisData`.
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp:258-260`
pub fn RE_SetWorldVisData(assets: &mut RenderAssets, vis: Vec<u8>) {
    assets.external_vis_data = Some(vis);
}

/// Raven `R_LoadVisibility`.
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp:268-296`
///
/// PORT-NOTE: the oracle's `Hunk_Alloc` + `Com_Memset`/`Com_Memcpy` pattern
/// collapses to direct owned-`Vec<u8>` allocation/copy throughout this file
/// — `WorldAsset`'s buffers are owned `Vec`s, not hunk-allocated raw
/// pointers, so the `Hunk_Alloc`/`Com_Mem*` seam calls have no idiomatic-
/// interior counterpart here.
fn R_LoadVisibility(
    ctx: &BspLoadContext,
    external_vis_data: Option<&Vec<u8>>,
    l: &lump_t,
    world: &mut WorldAsset,
) {
    let len = (world.num_clusters + 63) & !63;
    world.novis = vec![0xffu8; len.max(0) as usize];

    let filelen = l.filelen;
    if filelen == 0 {
        return;
    }
    let filelen = filelen as usize;

    let base = l.fileofs as usize;
    let buf = &ctx.file_base[base..];

    world.num_clusters = LittleLong(i32::from_le_bytes(buf[0..4].try_into().unwrap()));
    world.cluster_bytes = LittleLong(i32::from_le_bytes(buf[4..8].try_into().unwrap()));

    // CM_Load should have given us the vis data to share, so
    // we don't need to allocate another copy
    if let Some(external) = external_vis_data {
        world.vis = external.clone();
    } else {
        world.vis = buf[8..filelen].to_vec();
    }
}

/// Raven `R_MergedWidthPoints`.
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp:637-649`
pub fn R_MergedWidthPoints(grid: &GridMesh, offset: i32) -> bool {
    for i in 1..grid.width - 1 {
        for j in i + 1..grid.width - 1 {
            let a = grid.verts[(i + offset) as usize].xyz;
            let b = grid.verts[(j + offset) as usize].xyz;
            // `fabs()` takes a double: the f32 difference promotes and the `.1`
            // comparand is a double literal (ruling 12).
            if ((a[0] - b[0]) as f64).abs() > 0.1 {
                continue;
            }
            if ((a[1] - b[1]) as f64).abs() > 0.1 {
                continue;
            }
            if ((a[2] - b[2]) as f64).abs() > 0.1 {
                continue;
            }
            return true;
        }
    }
    false
}

/// Raven `R_MergedHeightPoints`.
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp:658-670`
pub fn R_MergedHeightPoints(grid: &GridMesh, offset: i32) -> bool {
    for i in 1..grid.height - 1 {
        for j in i + 1..grid.height - 1 {
            let a = grid.verts[(grid.width * i + offset) as usize].xyz;
            let b = grid.verts[(grid.width * j + offset) as usize].xyz;
            // `fabs()` takes a double: the f32 difference promotes and the `.1`
            // comparand is a double literal (ruling 12).
            if ((a[0] - b[0]) as f64).abs() > 0.1 {
                continue;
            }
            if ((a[1] - b[1]) as f64).abs() > 0.1 {
                continue;
            }
            if ((a[2] - b[2]) as f64).abs() > 0.1 {
                continue;
            }
            return true;
        }
    }
    false
}

/// Raven `R_SetParent` — recursively links a BSP tree node (and, if it is a
/// decision node, its children) to its parent.
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp:1478-1485`
fn R_SetParent(nodes: &mut [Node], node: usize, parent: Option<usize>) {
    nodes[node].parent = parent;
    if nodes[node].contents != -1 {
        return;
    }
    let children = nodes[node].children;
    if let Some(left) = children[0] {
        R_SetParent(nodes, left, Some(node));
    }
    if let Some(right) = children[1] {
        R_SetParent(nodes, right, Some(node));
    }
}

/// Raven `R_LoadShaders`.
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp:1570-1589`
fn R_LoadShaders(ctx: &BspLoadContext, l: &lump_t, world: &mut WorldAsset) {
    let entry_size = size_of::<dshader_t>();
    if (l.filelen as usize) % entry_size != 0 {
        com_error(
            errorParm_t::ERR_DROP,
            format!("LoadMap: funny lump size in {}", world.name),
        );
    }
    let count = l.filelen as usize / entry_size;
    let base = l.fileofs as usize;

    let mut shaders = Vec::with_capacity(count);
    for i in 0..count {
        let rec = &ctx.file_base[base + i * entry_size..base + (i + 1) * entry_size];
        let shader = latin1_name(&rec[0..MAX_QPATH]);
        let surface_flags = LittleLong(i32::from_le_bytes(rec[64..68].try_into().unwrap()));
        let content_flags = LittleLong(i32::from_le_bytes(rec[68..72].try_into().unwrap()));
        shaders.push(DShader {
            shader,
            surface_flags,
            content_flags,
        });
    }

    world.shaders = shaders;
}

/// Raven `R_LoadMarksurfaces`.
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp:1597-1617`
///
/// PORT-NOTE: the oracle stores `worldData.surfaces + j` — a pointer into
/// the surface array — per mark entry; the tier-2 transition audit's
/// replacement shape is the index `j` itself (`Vec<u32>` mark-index table),
/// so this port stores the parsed index directly rather than reconstructing
/// a pointer.
fn R_LoadMarksurfaces(ctx: &BspLoadContext, l: &lump_t, world: &mut WorldAsset) {
    let entry_size = size_of::<i32>();
    if (l.filelen as usize) % entry_size != 0 {
        com_error(
            errorParm_t::ERR_DROP,
            format!("LoadMap: funny lump size in {}", world.name),
        );
    }
    let count = l.filelen as usize / entry_size;
    let base = l.fileofs as usize;

    let mut mark_surfaces = Vec::with_capacity(count);
    for i in 0..count {
        let raw = i32::from_le_bytes(
            ctx.file_base[base + i * entry_size..base + (i + 1) * entry_size]
                .try_into()
                .unwrap(),
        );
        let j = LittleLong(raw);
        mark_surfaces.push(j as u32);
    }

    world.mark_surfaces = mark_surfaces;
}

/// Raven `R_LoadPlanes`.
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp:1625-1654`
///
/// PORT-NOTE: the oracle allocates `count * 2` planes (a later negative-
/// plane indexing trick unexercised by this loader) but only ever fills the
/// first `count`; the owned `Vec<cplane_t>` here holds exactly the `count`
/// parsed planes.
fn R_LoadPlanes(ctx: &BspLoadContext, l: &lump_t, world: &mut WorldAsset) {
    let entry_size = size_of::<dplane_t>();
    if (l.filelen as usize) % entry_size != 0 {
        com_error(
            errorParm_t::ERR_DROP,
            format!("LoadMap: funny lump size in {}", world.name),
        );
    }
    let count = l.filelen as usize / entry_size;
    let base = l.fileofs as usize;

    let mut planes = Vec::with_capacity(count);
    for i in 0..count {
        let rec = &ctx.file_base[base + i * entry_size..base + (i + 1) * entry_size];

        let mut normal = [0.0f32; 3];
        let mut bits = 0u8;
        for j in 0..3 {
            let f = LittleFloat(f32::from_le_bytes(
                rec[j * 4..j * 4 + 4].try_into().unwrap(),
            ));
            normal[j] = f;
            if f < 0.0 {
                bits |= 1 << j;
            }
        }
        let dist = LittleFloat(f32::from_le_bytes(rec[12..16].try_into().unwrap()));
        let plane_type = PlaneTypeForNormal(normal);

        planes.push(cplane_t {
            normal,
            dist,
            r#type: plane_type as u8,
            signbits: bits,
            pad: [0, 0],
        });
    }

    world.planes = planes;
}

/// Raven `R_LoadLightGridArray`.
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp:1856-1871`
pub fn R_LoadLightGridArray(
    common: &mut Common,
    ctx: &BspLoadContext,
    l: &lump_t,
    world: &mut WorldAsset,
) {
    world.num_grid_array_elements =
        world.light_grid_bounds[0] * world.light_grid_bounds[1] * world.light_grid_bounds[2];

    let expected = world.num_grid_array_elements as usize * size_of::<u16>();
    if l.filelen as usize != expected {
        // S_COLOR_YELLOW ("^3"), `mp_qshared::shared::q_color::S_COLOR_YELLOW`.
        com_printf(common, "^3WARNING: light grid array mismatch\n");
        world.light_grid_data = None;
        return;
    }

    let base = l.fileofs as usize;
    let bytes = &ctx.file_base[base..base + l.filelen as usize];
    let mut light_grid_array = Vec::with_capacity(world.num_grid_array_elements as usize);
    for chunk in bytes.chunks_exact(2) {
        light_grid_array.push(u16::from_le_bytes([chunk[0], chunk[1]]));
    }
    world.light_grid_array = light_grid_array;
}

/// Raven `R_GetEntityToken`.
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp:1978-1994`
///
/// PORT-NOTE: `buffer`/`size` are out-params in the oracle — translated to a
/// `(bool, String)` return per the out-params→returns dictionary entry.
/// `Q_strncpyz`'s raw `c_char` buffer copy collapses to a direct owned-
/// `String` truncation (Latin-1 discipline: bytes == chars); `size` reserves
/// one byte for the C null terminator Rust doesn't need, so the cap is
/// `size - 1`. The oracle's `!s_worldData.entityParsePoint` null check (the
/// parse cursor running off the end of the buffer) maps to the ported
/// `COM_Parse`'s remaining-slice going empty.
pub fn R_GetEntityToken(world: &mut WorldAsset, size: i32) -> (bool, String) {
    if size == -1 {
        // force reset
        world.entity_parse_point = 0;
        return (true, String::new());
    }

    let remaining = &world.entity_string[world.entity_parse_point..];
    let (token, rest) = COM_Parse(remaining, true);
    world.entity_parse_point = world.entity_string.len() - rest.len();

    let cap = (size.max(1) as usize).saturating_sub(1);
    let mut buffer = token.clone();
    if buffer.len() > cap {
        buffer.truncate(cap);
    }

    if rest.is_empty() || token.is_empty() {
        (false, buffer)
    } else {
        (true, buffer)
    }
}

// --- R3 wave 1 ---------------------------------------------------------

/// Borrows the grid at `world_data[a]` immutably and the one at
/// `world_data[b]` mutably from the same slice — the split-borrow helper
/// `R_FixSharedVertexLodError_r`'s recursion needs (its `grid1`/`grid2` alias
/// one array in the oracle, interior-safety law: pointer aliasing becomes an
/// index pair over one owned slice instead of two independent raw pointers).
/// `a == b` is unreachable here because the caller applies the oracle's own
/// `lodFixed == 2` guard (`tr_bsp.cpp:689-692`) before splitting: the parent
/// frame sets `grid2->lodFixed = 2` at `tr_bsp.cpp:777` *before* recursing
/// with that same grid as `grid1` at `:778`, so the recursive frame's loop
/// skips its own `grid1` index on the guard and never reaches this call with
/// `a == b`. It panics rather than silently aliasing if that ordering is ever
/// broken (porting-rules §19 — pick one defined behavior for what is
/// otherwise nonsensical input).
fn split_grid_pair(world_data: &mut [Surface], a: usize, b: usize) -> (&GridMesh, &mut GridMesh) {
    let (s1, s2) = if a < b {
        let (left, right) = world_data.split_at_mut(b);
        (&left[a].data, &mut right[0].data)
    } else {
        let (left, right) = world_data.split_at_mut(a);
        (&right[0].data, &mut left[b].data)
    };
    match (s1, s2) {
        (SurfaceData::Grid(grid1), SurfaceData::Grid(grid2)) => (grid1, grid2),
        // Unreachable: both callers apply the oracle's own `surfaceType !=
        // SF_GRID` guard to each index before splitting (porting-rules §19).
        _ => unreachable!("split_grid_pair on a non-grid surface"),
    }
}

/// Raven `R_FixSharedVertexLodError_r`.
///
/// PORT-NOTE: the packet's threading digest marks this "pure fn — no state
/// channel"; it walks `worldData.surfaces` as a plain `world_data: &mut
/// [Surface]` slice rather than the whole `WorldAsset` (DEC-43.1 gives the
/// carrier; this fn still needs nothing else off the world). `grid1` crosses
/// as an index into that same slice (`grid1_idx`) instead of a raw pointer:
/// the oracle's recursive call re-enters with `grid2` (an element of
/// `worldData.surfaces`) as the new `grid1`, aliasing the very array the loop
/// mutates, which Rust's aliasing rules forbid via two independent references
/// — `split_grid_pair` derives both from one `&mut` borrow per iteration
/// instead (interior-safety law: pointer → index).
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp:681-783`
pub fn R_FixSharedVertexLodError_r(start: usize, grid1_idx: usize, world_data: &mut [Surface]) {
    let mut j = start;
    while j < world_data.len() {
        let mut recurse = false;
        {
            // The two `grid2`-only guards read through plain immutable indexing
            // *before* the split borrow: the `lodFixed == 2` guard is what makes
            // `j == grid1_idx` unreachable at `split_grid_pair` (see its doc).
            // if this surface is not a grid
            let Some(grid2) = surface_grid(&world_data[j]) else {
                j += 1;
                continue;
            };
            // if the LOD errors are already fixed for this patch
            if grid2.lod_fixed == 2 {
                j += 1;
                continue;
            }

            let (grid1, grid2) = split_grid_pair(world_data, grid1_idx, j);

            // grids in the same LOD group should have the exact same lod radius
            if grid1.lod_radius != grid2.lod_radius {
                j += 1;
                continue;
            }
            // grids in the same LOD group should have the exact same lod origin
            if grid1.lod_origin[0] != grid2.lod_origin[0]
                || grid1.lod_origin[1] != grid2.lod_origin[1]
                || grid1.lod_origin[2] != grid2.lod_origin[2]
            {
                j += 1;
                continue;
            }

            //
            let mut touch = false;
            for n in 0..2i32 {
                //
                let offset1 = if n != 0 {
                    (grid1.height - 1) * grid1.width
                } else {
                    0
                };
                if R_MergedWidthPoints(grid1, offset1) {
                    continue;
                }
                for k in 1..(grid1.width - 1) {
                    for m in 0..2i32 {
                        let offset2 = if m != 0 {
                            (grid2.height - 1) * grid2.width
                        } else {
                            0
                        };
                        if R_MergedWidthPoints(grid2, offset2) {
                            continue;
                        }
                        for l in 1..(grid2.width - 1) {
                            //
                            let a = grid1.verts[(k + offset1) as usize].xyz;
                            let b = grid2.verts[(l + offset2) as usize].xyz;
                            // `fabs()` takes a double: the f32 difference promotes and the `.1`
                            // comparand is a double literal (ruling 12).
                            if ((a[0] - b[0]) as f64).abs() > 0.1 {
                                continue;
                            }
                            if ((a[1] - b[1]) as f64).abs() > 0.1 {
                                continue;
                            }
                            if ((a[2] - b[2]) as f64).abs() > 0.1 {
                                continue;
                            }
                            // ok the points are equal and should have the same lod error
                            grid2.width_lod_error[l as usize] = grid1.width_lod_error[k as usize];
                            touch = true;
                        }
                    }
                    for m in 0..2i32 {
                        let offset2 = if m != 0 { grid2.width - 1 } else { 0 };
                        if R_MergedHeightPoints(grid2, offset2) {
                            continue;
                        }
                        for l in 1..(grid2.height - 1) {
                            //
                            let a = grid1.verts[(k + offset1) as usize].xyz;
                            let b = grid2.verts[(grid2.width * l + offset2) as usize].xyz;
                            // `fabs()` takes a double: the f32 difference promotes and the `.1`
                            // comparand is a double literal (ruling 12).
                            if ((a[0] - b[0]) as f64).abs() > 0.1 {
                                continue;
                            }
                            if ((a[1] - b[1]) as f64).abs() > 0.1 {
                                continue;
                            }
                            if ((a[2] - b[2]) as f64).abs() > 0.1 {
                                continue;
                            }
                            // ok the points are equal and should have the same lod error
                            grid2.height_lod_error[l as usize] = grid1.width_lod_error[k as usize];
                            touch = true;
                        }
                    }
                }
            }
            for n in 0..2i32 {
                //
                let offset1 = if n != 0 { grid1.width - 1 } else { 0 };
                if R_MergedHeightPoints(grid1, offset1) {
                    continue;
                }
                for k in 1..(grid1.height - 1) {
                    for m in 0..2i32 {
                        let offset2 = if m != 0 {
                            (grid2.height - 1) * grid2.width
                        } else {
                            0
                        };
                        if R_MergedWidthPoints(grid2, offset2) {
                            continue;
                        }
                        for l in 1..(grid2.width - 1) {
                            //
                            let a = grid1.verts[(grid1.width * k + offset1) as usize].xyz;
                            let b = grid2.verts[(l + offset2) as usize].xyz;
                            // `fabs()` takes a double: the f32 difference promotes and the `.1`
                            // comparand is a double literal (ruling 12).
                            if ((a[0] - b[0]) as f64).abs() > 0.1 {
                                continue;
                            }
                            if ((a[1] - b[1]) as f64).abs() > 0.1 {
                                continue;
                            }
                            if ((a[2] - b[2]) as f64).abs() > 0.1 {
                                continue;
                            }
                            // ok the points are equal and should have the same lod error
                            grid2.width_lod_error[l as usize] = grid1.height_lod_error[k as usize];
                            touch = true;
                        }
                    }
                    for m in 0..2i32 {
                        let offset2 = if m != 0 { grid2.width - 1 } else { 0 };
                        if R_MergedHeightPoints(grid2, offset2) {
                            continue;
                        }
                        for l in 1..(grid2.height - 1) {
                            //
                            let a = grid1.verts[(grid1.width * k + offset1) as usize].xyz;
                            let b = grid2.verts[(grid2.width * l + offset2) as usize].xyz;
                            // `fabs()` takes a double: the f32 difference promotes and the `.1`
                            // comparand is a double literal (ruling 12).
                            if ((a[0] - b[0]) as f64).abs() > 0.1 {
                                continue;
                            }
                            if ((a[1] - b[1]) as f64).abs() > 0.1 {
                                continue;
                            }
                            if ((a[2] - b[2]) as f64).abs() > 0.1 {
                                continue;
                            }
                            // ok the points are equal and should have the same lod error
                            grid2.height_lod_error[l as usize] = grid1.height_lod_error[k as usize];
                            touch = true;
                        }
                    }
                }
            }
            if touch {
                grid2.lod_fixed = 2;
                //NOTE: this would be correct but makes things really slow
                //grid2->lodFixed = 1;
                recurse = true;
            }
        }
        if recurse {
            R_FixSharedVertexLodError_r(start, j, world_data);
        }
        j += 1;
    }
}

/// Raven `R_MovePatchSurfacesToHunk`.
///
/// PORT-NOTE: the oracle deep-copies each grid surface out of its original
/// (zone-scratch) allocation into a permanent hunk allocation — trimming
/// `widthLodError`/`heightLodError` to their exact size — then frees the
/// original via `R_FreeSurfaceGridMesh` and repoints `worldData.surfaces[i]
/// .data` at the hunk copy. Under the owned-`Vec` model every `GridMesh` is
/// already exactly-sized, permanently-owned storage (same collapse as
/// `R_LoadVisibility`'s Hunk_Alloc/Com_Mem* note above): there is no
/// separate hunk to move data into, so this reduces to the identity and
/// `R_FreeSurfaceGridMesh` (already ported, `tr_curve.rs`) is not called —
/// there is no separate original allocation for it to free. Also noted: the
/// oracle's `heightLodError` copy at `tr_bsp.cpp:1333` copies
/// `grid->heightLodError` onto itself (`grid->heightLodError,
/// grid->heightLodError`, not `hunkgrid->heightLodError`) — an oracle bug
/// that leaves `hunkgrid->heightLodError` as uninitialized hunk memory in
/// the original. Porting-rules §19: picking the defined behavior of
/// preserving the real `heightLodError` values (there is no "uninitialized
/// hunk memory" concept to reproduce under owned `Vec`s) rather than
/// modeling that corruption.
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp:1314-1339`
pub fn R_MovePatchSurfacesToHunk(_world_data: &mut [Surface]) {
    // No-op under the owned-Vec ownership model — see PORT-NOTE above.
}

/// Raven `R_LoadNodesAndLeafs`.
///
/// PORT-NOTE: `worldData.nodes`'s parent/children pointer graph becomes the
/// index-linked `Node` arena (tier-2 transition audit, Group 1: `mnode_t`
/// row); the oracle's single `Hunk_Alloc`'d `(numNodes + numLeafs)`-element
/// array collapses to one `Vec<Node>` built directly from the parsed file
/// bytes (same Hunk_Alloc/Com_Mem* collapse as `R_LoadVisibility` above —
/// no idiomatic-interior counterpart for the hunk allocator). `plane`
/// crosses as an index into `world.planes`; `firstmarksurface` as an index
/// into `world.mark_surfaces` (both already-populated by
/// `R_LoadPlanes`/`R_LoadMarksurfaces`, lower-wave siblings in this file).
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp:1492-1561`
fn R_LoadNodesAndLeafs(
    ctx: &BspLoadContext,
    node_lump: &lump_t,
    leaf_lump: &lump_t,
    world: &mut WorldAsset,
) {
    let node_entry_size = size_of::<dnode_t>();
    let leaf_entry_size = size_of::<dleaf_t>();
    if (node_lump.filelen as usize) % node_entry_size != 0
        || (leaf_lump.filelen as usize) % leaf_entry_size != 0
    {
        com_error(
            errorParm_t::ERR_DROP,
            format!("LoadMap: funny lump size in {}", world.name),
        );
    }

    let num_nodes = node_lump.filelen as usize / node_entry_size;
    let num_leafs = leaf_lump.filelen as usize / leaf_entry_size;

    let mut nodes: Vec<Node> = Vec::with_capacity(num_nodes + num_leafs);

    // load nodes
    let node_base = node_lump.fileofs as usize;
    for i in 0..num_nodes {
        let rec =
            &ctx.file_base[node_base + i * node_entry_size..node_base + (i + 1) * node_entry_size];

        let mut mins = [0i32; 3];
        let mut maxs = [0i32; 3];
        for j in 0..3 {
            mins[j] = LittleLong(i32::from_le_bytes(
                rec[12 + j * 4..16 + j * 4].try_into().unwrap(),
            ));
            maxs[j] = LittleLong(i32::from_le_bytes(
                rec[24 + j * 4..28 + j * 4].try_into().unwrap(),
            ));
        }

        let plane_num = LittleLong(i32::from_le_bytes(rec[0..4].try_into().unwrap()));

        let mut children = [None, None];
        for j in 0..2 {
            let p = LittleLong(i32::from_le_bytes(
                rec[4 + j * 4..8 + j * 4].try_into().unwrap(),
            ));
            children[j] = if p >= 0 {
                Some(p as usize)
            } else {
                Some(num_nodes + (-1 - p) as usize)
            };
        }

        nodes.push(Node {
            parent: None,
            children,
            contents: -1, // CONTENTS_NODE — differentiate from leafs
            mins,
            maxs,
            plane: Some(plane_num as usize),
            cluster: 0,
            area: 0,
            firstmarksurface: 0,
            nummarksurfaces: 0,
        });
    }

    // load leafs
    let leaf_base = leaf_lump.fileofs as usize;
    for i in 0..num_leafs {
        let rec =
            &ctx.file_base[leaf_base + i * leaf_entry_size..leaf_base + (i + 1) * leaf_entry_size];

        let mut mins = [0i32; 3];
        let mut maxs = [0i32; 3];
        for j in 0..3 {
            mins[j] = LittleLong(i32::from_le_bytes(
                rec[8 + j * 4..12 + j * 4].try_into().unwrap(),
            ));
            maxs[j] = LittleLong(i32::from_le_bytes(
                rec[20 + j * 4..24 + j * 4].try_into().unwrap(),
            ));
        }

        let cluster = LittleLong(i32::from_le_bytes(rec[0..4].try_into().unwrap()));
        let area = LittleLong(i32::from_le_bytes(rec[4..8].try_into().unwrap()));

        if cluster >= world.num_clusters {
            world.num_clusters = cluster + 1;
        }

        let first_leaf_surface = LittleLong(i32::from_le_bytes(rec[32..36].try_into().unwrap()));
        let num_leaf_surfaces = LittleLong(i32::from_le_bytes(rec[36..40].try_into().unwrap()));

        nodes.push(Node {
            parent: None,
            children: [None, None],
            // Raven never assigns `out->contents` in this leaf loop; leafs
            // keep the zero-initialized default (see the `Node::contents`
            // doc comment).
            contents: 0,
            mins,
            maxs,
            plane: None,
            cluster,
            area,
            firstmarksurface: first_leaf_surface as usize,
            nummarksurfaces: num_leaf_surfaces,
        });
    }

    // chain decendants
    R_SetParent(&mut nodes, 0, None);

    world.num_decision_nodes = num_nodes as i32;
    world.nodes = nodes;
}

/// Raven `R_LoadLightGrid`.
///
/// PORT-NOTE: `wave-0 ruling 12` — `1.0 / w->lightGridSize[i]` (a C `double`
/// literal divided by `float`) and the `ceil`/`floor` calls (which promote
/// their `float` argument to `double`, the standard-library signature)
/// evaluate in `f64` here, rounded to `f32` once at the C assignment point.
/// The `lightGridBounds[i]` line has no double literal or `ceil`/`floor`
/// call — every operand is `float` — so it stays plain `f32` arithmetic.
/// `w->lightGridData`'s `Hunk_Alloc` + `memcpy` collapses to direct
/// `Vec<mgrid_t>` construction (same collapse as `R_LoadVisibility` above);
/// `mgrid_t`'s fields are all byte arrays, so no `LittleLong`/`LittleFloat`
/// endian conversion applies. The 3-byte-in-place
/// `R_ColorShiftLightingBytes` overload is `R_ColorShiftLightingBytesRGB`
/// (this file, wave 0) — its Rust shape returns the shifted bytes instead of
/// mutating in place.
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp:1813-1848`
pub fn R_LoadLightGrid(
    ctx: &BspLoadContext,
    world_load: &WorldLoadState,
    l: &lump_t,
    world: &mut WorldAsset,
) {
    for i in 0..3usize {
        world.light_grid_inverse_size[i] = (1.0f64 / world.light_grid_size[i] as f64) as f32;
    }

    let w_mins = world.bmodels[0].bounds[0];
    let w_maxs = world.bmodels[0].bounds[1];

    let mut maxs = [0.0f32; 3];
    for i in 0..3usize {
        world.light_grid_origin[i] = (world.light_grid_size[i] as f64
            * (w_mins[i] as f64 / world.light_grid_size[i] as f64).ceil())
            as f32;
        maxs[i] = (world.light_grid_size[i] as f64
            * (w_maxs[i] as f64 / world.light_grid_size[i] as f64).floor())
            as f32;
        world.light_grid_bounds[i] =
            ((maxs[i] - world.light_grid_origin[i]) / world.light_grid_size[i] + 1.0) as i32;
    }

    let entry_size = size_of::<mgrid_t>();
    let num_grid_data_elements = l.filelen as usize / entry_size;

    let base = l.fileofs as usize;
    let mut light_grid_data = Vec::with_capacity(num_grid_data_elements);
    for i in 0..num_grid_data_elements {
        let rec = &ctx.file_base[base + i * entry_size..base + (i + 1) * entry_size];

        let mut ambient_light = [[0u8; 3]; MAXLIGHTMAPS];
        for k in 0..MAXLIGHTMAPS {
            ambient_light[k] = [rec[k * 3], rec[k * 3 + 1], rec[k * 3 + 2]];
        }
        let direct_base = MAXLIGHTMAPS * 3;
        let mut direct_light = [[0u8; 3]; MAXLIGHTMAPS];
        for k in 0..MAXLIGHTMAPS {
            direct_light[k] = [
                rec[direct_base + k * 3],
                rec[direct_base + k * 3 + 1],
                rec[direct_base + k * 3 + 2],
            ];
        }
        let styles_base = direct_base + MAXLIGHTMAPS * 3;
        let mut styles = [0u8; MAXLIGHTMAPS];
        styles.copy_from_slice(&rec[styles_base..styles_base + MAXLIGHTMAPS]);
        let lat_long_base = styles_base + MAXLIGHTMAPS;
        let lat_long = [rec[lat_long_base], rec[lat_long_base + 1]];

        let mut grid = mgrid_t {
            ambientLight: ambient_light,
            directLight: direct_light,
            styles,
            latLong: lat_long,
        };

        // deal with overbright bits
        for j in 0..MAXLIGHTMAPS {
            grid.ambientLight[j] = R_ColorShiftLightingBytesRGB(world_load, grid.ambientLight[j]);
            grid.directLight[j] = R_ColorShiftLightingBytesRGB(world_load, grid.directLight[j]);
        }

        light_grid_data.push(grid);
    }

    world.light_grid_data = Some(light_grid_data);
}

// --- R3 wave 2 ---------------------------------------------------------

/// Raven `R_FixSharedVertexLodError`.
///
/// PORT-NOTE: same "no state channel" / plain-`world_data` shape as the
/// wave-1 sibling `R_FixSharedVertexLodError_r` it drives — it walks
/// `worldData.surfaces` as a `&mut [Surface]` slice (DEC-43.1) and needs
/// nothing else off `WorldAsset`.
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp:793-811`
pub fn R_FixSharedVertexLodError(world_data: &mut [Surface]) {
    for i in 0..world_data.len() {
        //
        // if this surface is not a grid
        let Some(grid) = surface_grid_mut(&mut world_data[i]) else {
            continue;
        };
        //
        if grid.lod_fixed != 0 {
            continue;
        }
        //
        grid.lod_fixed = 2;
        // recursively fix other patches in the same LOD group
        R_FixSharedVertexLodError_r(i + 1, i, world_data);
    }
}

/// Raven's repeated `if (fabs(v1[i] - v2[i]) > .1) continue;` triple — true
/// when any component differs by more than `tol`. `fabs()` takes a double:
/// the f32 difference promotes and the comparand is a double literal
/// (ruling 12).
fn vectors_differ(v1: Vec3, v2: Vec3, tol: f64) -> bool {
    ((v1[0] - v2[0]) as f64).abs() > tol
        || ((v1[1] - v2[1]) as f64).abs() > tol
        || ((v1[2] - v2[2]) as f64).abs() > tol
}

/// Raven's `fabs(v1[0]-v2[0]) < .01 && fabs(v1[1]-v2[1]) < .01 &&
/// fabs(v1[2]-v2[2]) < .01` conjunction — every component within `tol`. Kept
/// distinct from `vectors_differ` because the strict `<` and `>` forms differ
/// exactly at `tol`.
fn vectors_coincide(v1: Vec3, v2: Vec3, tol: f64) -> bool {
    ((v1[0] - v2[0]) as f64).abs() < tol
        && ((v1[1] - v2[1]) as f64).abs() < tol
        && ((v1[2] - v2[2]) as f64).abs() < tol
}

/// Reads `grid1->widthLodError[k+1]` / `heightLodError[k+1]` at the insert
/// sites of `R_StitchPatches`'s two descending-`k` passes
/// (`oracle/codemp/renderer/tr_bsp.cpp:1029-1233`), where `k` starts at
/// `width-1`/`height-1` — so `k+1` indexes one past the `width`/`height`-entry
/// table on the first iteration, a C heap over-read (porting-rules §19: the
/// defined behavior picked here is the last valid entry). The ascending-`k`
/// passes stay in range and index the table directly.
fn lod_error_clamped(table: &[f32], index: i32) -> f32 {
    let last = table.len().saturating_sub(1);
    table[(index.max(0) as usize).min(last)]
}

/// The grid edit `R_StitchPatches` found — Raven's
/// `R_GridInsertColumn`/`R_GridInsertRow` call plus its already-evaluated
/// arguments, read off `grid1` at the oracle's own read point (before the
/// insert reshapes `grid2`).
enum StitchInsert {
    Column {
        column: usize,
        row: usize,
        point: Vec3,
        loderror: f32,
    },
    Row {
        row: usize,
        column: usize,
        point: Vec3,
        loderror: f32,
    },
}

/// The read-only search half of Raven `R_StitchPatches` — every one of the
/// oracle's eight insert sites is immediately followed by `return qtrue`, so
/// the whole scan runs before any mutation and can hand its single edit back
/// to the caller (porting-rules C10: control-flow shape may change).
/// `grid1`/`grid2` are two shared borrows of one slice rather than
/// `split_grid_pair`'s `&`/`&mut` split: nothing is written here, and
/// `R_TryStitchingPatch` (`tr_bsp.cpp:1247-1262`) does call this with
/// `grid1num == j`.
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp:819-1234`
fn stitch_scan(grid1: &GridMesh, grid2: &GridMesh) -> Option<StitchInsert> {
    for n in 0..2i32 {
        //
        let offset1 = if n != 0 {
            (grid1.height - 1) * grid1.width
        } else {
            0
        };
        if R_MergedWidthPoints(grid1, offset1) {
            continue;
        }
        let mut k = 0i32;
        while k < grid1.width - 2 {
            for m in 0..2i32 {
                if grid2.width >= MAX_GRID_SIZE as i32 {
                    break;
                }
                let offset2 = if m != 0 {
                    (grid2.height - 1) * grid2.width
                } else {
                    0
                };
                //if (R_MergedWidthPoints(grid2, offset2))
                //	continue;
                for l in 0..grid2.width - 1 {
                    //
                    let v1 = grid1.verts[(k + offset1) as usize].xyz;
                    let v2 = grid2.verts[(l + offset2) as usize].xyz;
                    if vectors_differ(v1, v2, 0.1) {
                        continue;
                    }

                    let v1 = grid1.verts[(k + 2 + offset1) as usize].xyz;
                    let v2 = grid2.verts[(l + 1 + offset2) as usize].xyz;
                    if vectors_differ(v1, v2, 0.1) {
                        continue;
                    }
                    //
                    let v1 = grid2.verts[(l + offset2) as usize].xyz;
                    let v2 = grid2.verts[(l + 1 + offset2) as usize].xyz;
                    if vectors_coincide(v1, v2, 0.01) {
                        continue;
                    }
                    //
                    //Com_Printf ("found highest LoD crack between two patches\n" );
                    // insert column into grid2 right after after column l
                    let row = if m != 0 { grid2.height - 1 } else { 0 };
                    return Some(StitchInsert::Column {
                        column: (l + 1) as usize,
                        row: row as usize,
                        point: grid1.verts[(k + 1 + offset1) as usize].xyz,
                        loderror: grid1.width_lod_error[(k + 1) as usize],
                    });
                }
            }
            for m in 0..2i32 {
                if grid2.height >= MAX_GRID_SIZE as i32 {
                    break;
                }
                let offset2 = if m != 0 { grid2.width - 1 } else { 0 };
                //if (R_MergedHeightPoints(grid2, offset2))
                //	continue;
                for l in 0..grid2.height - 1 {
                    //
                    let v1 = grid1.verts[(k + offset1) as usize].xyz;
                    let v2 = grid2.verts[(grid2.width * l + offset2) as usize].xyz;
                    if vectors_differ(v1, v2, 0.1) {
                        continue;
                    }

                    let v1 = grid1.verts[(k + 2 + offset1) as usize].xyz;
                    let v2 = grid2.verts[(grid2.width * (l + 1) + offset2) as usize].xyz;
                    if vectors_differ(v1, v2, 0.1) {
                        continue;
                    }
                    //
                    let v1 = grid2.verts[(grid2.width * l + offset2) as usize].xyz;
                    let v2 = grid2.verts[(grid2.width * (l + 1) + offset2) as usize].xyz;
                    if vectors_coincide(v1, v2, 0.01) {
                        continue;
                    }
                    //
                    //Com_Printf ("found highest LoD crack between two patches\n" );
                    // insert row into grid2 right after after row l
                    let column = if m != 0 { grid2.width - 1 } else { 0 };
                    return Some(StitchInsert::Row {
                        row: (l + 1) as usize,
                        column: column as usize,
                        point: grid1.verts[(k + 1 + offset1) as usize].xyz,
                        loderror: grid1.width_lod_error[(k + 1) as usize],
                    });
                }
            }
            k += 2;
        }
    }
    for n in 0..2i32 {
        //
        let offset1 = if n != 0 { grid1.width - 1 } else { 0 };
        if R_MergedHeightPoints(grid1, offset1) {
            continue;
        }
        let mut k = 0i32;
        while k < grid1.height - 2 {
            for m in 0..2i32 {
                if grid2.width >= MAX_GRID_SIZE as i32 {
                    break;
                }
                let offset2 = if m != 0 {
                    (grid2.height - 1) * grid2.width
                } else {
                    0
                };
                //if (R_MergedWidthPoints(grid2, offset2))
                //	continue;
                for l in 0..grid2.width - 1 {
                    //
                    let v1 = grid1.verts[(grid1.width * k + offset1) as usize].xyz;
                    let v2 = grid2.verts[(l + offset2) as usize].xyz;
                    if vectors_differ(v1, v2, 0.1) {
                        continue;
                    }

                    let v1 = grid1.verts[(grid1.width * (k + 2) + offset1) as usize].xyz;
                    let v2 = grid2.verts[(l + 1 + offset2) as usize].xyz;
                    if vectors_differ(v1, v2, 0.1) {
                        continue;
                    }
                    //
                    let v1 = grid2.verts[(l + offset2) as usize].xyz;
                    let v2 = grid2.verts[((l + 1) + offset2) as usize].xyz;
                    if vectors_coincide(v1, v2, 0.01) {
                        continue;
                    }
                    //
                    //Com_Printf ("found highest LoD crack between two patches\n" );
                    // insert column into grid2 right after after column l
                    let row = if m != 0 { grid2.height - 1 } else { 0 };
                    return Some(StitchInsert::Column {
                        column: (l + 1) as usize,
                        row: row as usize,
                        point: grid1.verts[(grid1.width * (k + 1) + offset1) as usize].xyz,
                        loderror: grid1.height_lod_error[(k + 1) as usize],
                    });
                }
            }
            for m in 0..2i32 {
                if grid2.height >= MAX_GRID_SIZE as i32 {
                    break;
                }
                let offset2 = if m != 0 { grid2.width - 1 } else { 0 };
                //if (R_MergedHeightPoints(grid2, offset2))
                //	continue;
                for l in 0..grid2.height - 1 {
                    //
                    let v1 = grid1.verts[(grid1.width * k + offset1) as usize].xyz;
                    let v2 = grid2.verts[(grid2.width * l + offset2) as usize].xyz;
                    if vectors_differ(v1, v2, 0.1) {
                        continue;
                    }

                    let v1 = grid1.verts[(grid1.width * (k + 2) + offset1) as usize].xyz;
                    let v2 = grid2.verts[(grid2.width * (l + 1) + offset2) as usize].xyz;
                    if vectors_differ(v1, v2, 0.1) {
                        continue;
                    }
                    //
                    let v1 = grid2.verts[(grid2.width * l + offset2) as usize].xyz;
                    let v2 = grid2.verts[(grid2.width * (l + 1) + offset2) as usize].xyz;
                    if vectors_coincide(v1, v2, 0.01) {
                        continue;
                    }
                    //
                    //Com_Printf ("found highest LoD crack between two patches\n" );
                    // insert row into grid2 right after after row l
                    let column = if m != 0 { grid2.width - 1 } else { 0 };
                    return Some(StitchInsert::Row {
                        row: (l + 1) as usize,
                        column: column as usize,
                        point: grid1.verts[(grid1.width * (k + 1) + offset1) as usize].xyz,
                        loderror: grid1.height_lod_error[(k + 1) as usize],
                    });
                }
            }
            k += 2;
        }
    }
    for n in 0..2i32 {
        //
        let offset1 = if n != 0 {
            (grid1.height - 1) * grid1.width
        } else {
            0
        };
        if R_MergedWidthPoints(grid1, offset1) {
            continue;
        }
        let mut k = grid1.width - 1;
        while k > 1 {
            for m in 0..2i32 {
                if grid2.width >= MAX_GRID_SIZE as i32 {
                    break;
                }
                let offset2 = if m != 0 {
                    (grid2.height - 1) * grid2.width
                } else {
                    0
                };
                //if (R_MergedWidthPoints(grid2, offset2))
                //	continue;
                for l in 0..grid2.width - 1 {
                    //
                    let v1 = grid1.verts[(k + offset1) as usize].xyz;
                    let v2 = grid2.verts[(l + offset2) as usize].xyz;
                    if vectors_differ(v1, v2, 0.1) {
                        continue;
                    }

                    let v1 = grid1.verts[(k - 2 + offset1) as usize].xyz;
                    let v2 = grid2.verts[(l + 1 + offset2) as usize].xyz;
                    if vectors_differ(v1, v2, 0.1) {
                        continue;
                    }
                    //
                    let v1 = grid2.verts[(l + offset2) as usize].xyz;
                    let v2 = grid2.verts[((l + 1) + offset2) as usize].xyz;
                    if vectors_coincide(v1, v2, 0.01) {
                        continue;
                    }
                    //
                    //Com_Printf ("found highest LoD crack between two patches\n" );
                    // insert column into grid2 right after after column l
                    let row = if m != 0 { grid2.height - 1 } else { 0 };
                    return Some(StitchInsert::Column {
                        column: (l + 1) as usize,
                        row: row as usize,
                        point: grid1.verts[(k - 1 + offset1) as usize].xyz,
                        loderror: lod_error_clamped(&grid1.width_lod_error, k + 1),
                    });
                }
            }
            for m in 0..2i32 {
                if grid2.height >= MAX_GRID_SIZE as i32 {
                    break;
                }
                let offset2 = if m != 0 { grid2.width - 1 } else { 0 };
                //if (R_MergedHeightPoints(grid2, offset2))
                //	continue;
                for l in 0..grid2.height - 1 {
                    //
                    let v1 = grid1.verts[(k + offset1) as usize].xyz;
                    let v2 = grid2.verts[(grid2.width * l + offset2) as usize].xyz;
                    if vectors_differ(v1, v2, 0.1) {
                        continue;
                    }

                    let v1 = grid1.verts[(k - 2 + offset1) as usize].xyz;
                    let v2 = grid2.verts[(grid2.width * (l + 1) + offset2) as usize].xyz;
                    if vectors_differ(v1, v2, 0.1) {
                        continue;
                    }
                    //
                    let v1 = grid2.verts[(grid2.width * l + offset2) as usize].xyz;
                    let v2 = grid2.verts[(grid2.width * (l + 1) + offset2) as usize].xyz;
                    if vectors_coincide(v1, v2, 0.01) {
                        continue;
                    }
                    //
                    //Com_Printf ("found highest LoD crack between two patches\n" );
                    // insert row into grid2 right after after row l
                    let column = if m != 0 { grid2.width - 1 } else { 0 };
                    // Raven's lone `if (!grid2) break;` null check on the
                    // insert result lives here (`tr_bsp.cpp:1124`); it is
                    // honoured by `R_StitchPatches`'s pre-insert capacity
                    // check, uniformly for all eight sites.
                    return Some(StitchInsert::Row {
                        row: (l + 1) as usize,
                        column: column as usize,
                        point: grid1.verts[(k - 1 + offset1) as usize].xyz,
                        loderror: lod_error_clamped(&grid1.width_lod_error, k + 1),
                    });
                }
            }
            k -= 2;
        }
    }
    for n in 0..2i32 {
        //
        let offset1 = if n != 0 { grid1.width - 1 } else { 0 };
        if R_MergedHeightPoints(grid1, offset1) {
            continue;
        }
        let mut k = grid1.height - 1;
        while k > 1 {
            for m in 0..2i32 {
                if grid2.width >= MAX_GRID_SIZE as i32 {
                    break;
                }
                let offset2 = if m != 0 {
                    (grid2.height - 1) * grid2.width
                } else {
                    0
                };
                //if (R_MergedWidthPoints(grid2, offset2))
                //	continue;
                for l in 0..grid2.width - 1 {
                    //
                    let v1 = grid1.verts[(grid1.width * k + offset1) as usize].xyz;
                    let v2 = grid2.verts[(l + offset2) as usize].xyz;
                    if vectors_differ(v1, v2, 0.1) {
                        continue;
                    }

                    let v1 = grid1.verts[(grid1.width * (k - 2) + offset1) as usize].xyz;
                    let v2 = grid2.verts[(l + 1 + offset2) as usize].xyz;
                    if vectors_differ(v1, v2, 0.1) {
                        continue;
                    }
                    //
                    let v1 = grid2.verts[(l + offset2) as usize].xyz;
                    let v2 = grid2.verts[((l + 1) + offset2) as usize].xyz;
                    if vectors_coincide(v1, v2, 0.01) {
                        continue;
                    }
                    //
                    //Com_Printf ("found highest LoD crack between two patches\n" );
                    // insert column into grid2 right after after column l
                    let row = if m != 0 { grid2.height - 1 } else { 0 };
                    return Some(StitchInsert::Column {
                        column: (l + 1) as usize,
                        row: row as usize,
                        point: grid1.verts[(grid1.width * (k - 1) + offset1) as usize].xyz,
                        loderror: lod_error_clamped(&grid1.height_lod_error, k + 1),
                    });
                }
            }
            for m in 0..2i32 {
                if grid2.height >= MAX_GRID_SIZE as i32 {
                    break;
                }
                let offset2 = if m != 0 { grid2.width - 1 } else { 0 };
                //if (R_MergedHeightPoints(grid2, offset2))
                //	continue;
                for l in 0..grid2.height - 1 {
                    //
                    let v1 = grid1.verts[(grid1.width * k + offset1) as usize].xyz;
                    let v2 = grid2.verts[(grid2.width * l + offset2) as usize].xyz;
                    if vectors_differ(v1, v2, 0.1) {
                        continue;
                    }

                    let v1 = grid1.verts[(grid1.width * (k - 2) + offset1) as usize].xyz;
                    let v2 = grid2.verts[(grid2.width * (l + 1) + offset2) as usize].xyz;
                    if vectors_differ(v1, v2, 0.1) {
                        continue;
                    }
                    //
                    let v1 = grid2.verts[(grid2.width * l + offset2) as usize].xyz;
                    let v2 = grid2.verts[(grid2.width * (l + 1) + offset2) as usize].xyz;
                    if vectors_coincide(v1, v2, 0.01) {
                        continue;
                    }
                    //
                    //Com_Printf ("found highest LoD crack between two patches\n" );
                    // insert row into grid2 right after after row l
                    let column = if m != 0 { grid2.width - 1 } else { 0 };
                    return Some(StitchInsert::Row {
                        row: (l + 1) as usize,
                        column: column as usize,
                        point: grid1.verts[(grid1.width * (k - 1) + offset1) as usize].xyz,
                        loderror: lod_error_clamped(&grid1.height_lod_error, k + 1),
                    });
                }
            }
            k -= 2;
        }
    }
    None
}

/// Raven `R_StitchPatches` — fixes one highest-LOD crack between two patches
/// in the same LOD group, returning whether it changed anything.
///
/// `R_GridInsertColumn`/`R_GridInsertRow` (`tr_curve.rs`) take the grid by
/// value, so `grid2` is moved out of its `SurfaceData::Grid` slot with
/// `core::mem::replace` and the returned grid moved back — the owned form of
/// the oracle's `worldData.surfaces[grid2num].data = (surfaceType_t *) grid2;`
/// repoint. The transient placeholder left in the slot is
/// `empty_grid_mesh()` (tag `SF_BAD`), not `SurfaceData::Skip`: `SF_SKIP` is
/// a meaningful nodraw surface, while `SF_BAD` is self-evidently a hole.
///
/// PORT-NOTE: same "no state channel" / plain-`world_data` shape as its
/// `R_FixSharedVertexLodError` siblings above — it walks `worldData.surfaces`
/// as a `&mut [Surface]` slice (DEC-43.1).
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp:819-1235`
pub fn R_StitchPatches(grid1num: usize, grid2num: usize, world_data: &mut [Surface]) -> bool {
    let (Some(grid1), Some(grid2)) = (
        surface_grid(&world_data[grid1num]),
        surface_grid(&world_data[grid2num]),
    ) else {
        // Unreachable: `R_StitchAllPatches`/`R_TryStitchingPatch` apply the
        // oracle's `surfaceType != SF_GRID` guard to both indices first.
        unreachable!("R_StitchPatches on a non-grid surface")
    };
    let insert = stitch_scan(grid1, grid2);
    let Some(insert) = insert else {
        return false;
    };

    // The callee bails (`None`) exactly when the grid is already
    // `MAX_GRID_SIZE` wide/tall, which `stitch_scan`'s own `>= MAX_GRID_SIZE`
    // guards already exclude; re-checked before the move so a bail can never
    // consume the grid out of its slot.
    let fits = match insert {
        StitchInsert::Column { .. } => grid2.width < MAX_GRID_SIZE as i32,
        StitchInsert::Row { .. } => grid2.height < MAX_GRID_SIZE as i32,
    };
    if !fits {
        return false;
    }

    let SurfaceData::Grid(grid2) = replace(
        &mut world_data[grid2num].data,
        SurfaceData::Grid(empty_grid_mesh()),
    ) else {
        unreachable!("R_StitchPatches on a non-grid surface")
    };
    let stitched = match insert {
        StitchInsert::Column {
            column,
            row,
            point,
            loderror,
        } => R_GridInsertColumn(grid2, column, row, point, loderror),
        StitchInsert::Row {
            row,
            column,
            point,
            loderror,
        } => R_GridInsertRow(grid2, row, column, point, loderror),
    };

    match stitched {
        Some(mut grid2) => {
            grid2.lod_stitched = 0;
            world_data[grid2num].data = SurfaceData::Grid(grid2);
            true
        }
        // Unreachable — see the `fits` check above. The callee consumed the
        // grid on this path, so the slot keeps its `SF_BAD` placeholder.
        None => false,
    }
}

/// Raven `R_LoadSubmodels`.
///
/// PORT-NOTE: the lump-parsing half (bounds/`firstSurface`/`numSurfaces` per
/// submodel, feeding `WorldAsset::bmodels`) needs no tier-2 access. The
/// per-submodel `model_t` registration half runs through
/// `RenderModels::register_bmodel` (`tr_model/render_models.rs`), the
/// `pub(crate)` wrapper over `r_alloc_model`/`re_insert_model_into_hash`. It
/// records the handle against its `WorldAsset::bmodels` index in a side map
/// rather than writing the retired `model_t::bmodel` raw pointer, so no
/// tier-2 pointer is constructed here (interior-safety law: "UNSAFE IS BANNED
/// IN THIS FILE"). Registration runs in a second pass after `world.bmodels`
/// is fully populated, so a handle always resolves to a parsed row; the
/// allocation order (`i = 0..count`) matches the oracle's interleaved loop.
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp:1421-1467`
fn R_LoadSubmodels(
    ctx: &BspLoadContext,
    l: &lump_t,
    world: &mut WorldAsset,
    models: &mut RenderModels,
    index: i32,
) {
    let entry_size = size_of::<dmodel_t>();
    if (l.filelen as usize) % entry_size != 0 {
        com_error(
            errorParm_t::ERR_DROP,
            format!("LoadMap: funny lump size in {}", world.name),
        );
    }
    let count = l.filelen as usize / entry_size;
    let base = l.fileofs as usize;

    let mut bmodels = Vec::with_capacity(count);
    for i in 0..count {
        let rec = &ctx.file_base[base + i * entry_size..base + (i + 1) * entry_size];

        let mut mins = [0.0f32; 3];
        let mut maxs = [0.0f32; 3];
        for j in 0..3 {
            mins[j] = LittleFloat(f32::from_le_bytes(
                rec[j * 4..j * 4 + 4].try_into().unwrap(),
            ));
            maxs[j] = LittleFloat(f32::from_le_bytes(
                rec[12 + j * 4..16 + j * 4].try_into().unwrap(),
            ));
        }
        let first_surface = LittleLong(i32::from_le_bytes(rec[24..28].try_into().unwrap()));
        let num_surfaces = LittleLong(i32::from_le_bytes(rec[28..32].try_into().unwrap()));

        bmodels.push(BModel {
            bounds: [mins, maxs],
            first_surface: first_surface as usize,
            num_surfaces,
        });
    }

    world.bmodels = bmodels;

    // Register a `model_t` for each parsed submodel. The oracle interleaves
    // this with the bounds parse above; running it as a second pass over the
    // fully populated `world.bmodels` is behaviorally identical (same
    // allocation order i = 0..count) and keeps the parse free of the model
    // registry.
    // Source: oracle/codemp/renderer/tr_bsp.cpp:1433-1463
    for i in 0..count {
        models.register_bmodel(i, index);
    }
}

// --- R3 wave 3 ---------------------------------------------------------

/// Raven `R_TryStitchingPatch`.
///
/// PORT-NOTE: same "no state channel" / plain-`world_data` shape as its
/// `R_StitchPatches`/`R_FixSharedVertexLodError` siblings above — it walks
/// `worldData.surfaces` as a `&mut [Surface]` slice (DEC-43.1);
/// `worldData.numsurfaces` is `world_data.len()`. The oracle caches
/// `grid1 = worldData.surfaces[grid1num].data` once before the loop and never
/// re-fetches it, but `lodRadius`/`lodOrigin` (the only fields read off
/// `grid1` here) are never mutated by `R_StitchPatches`, so re-indexing
/// `world_data[grid1num]` fresh each iteration is behaviorally identical and
/// avoids holding a stale reference across the mutating call
/// (interior-safety law: no raw pointers to alias here in the first place).
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp:1250-1274`
pub fn R_TryStitchingPatch(grid1num: usize, world_data: &mut [Surface]) -> i32 {
    let mut numstitches = 0i32;
    for j in 0..world_data.len() {
        //
        // if this surface is not a grid
        let Some(grid2) = surface_grid(&world_data[j]) else {
            continue;
        };
        let Some(grid1) = surface_grid(&world_data[grid1num]) else {
            // Unreachable: `R_StitchAllPatches` only calls in with a grid.
            unreachable!("R_TryStitchingPatch on a non-grid surface")
        };
        // grids in the same LOD group should have the exact same lod radius
        if grid1.lod_radius != grid2.lod_radius {
            continue;
        }
        // grids in the same LOD group should have the exact same lod origin
        if grid1.lod_origin[0] != grid2.lod_origin[0] {
            continue;
        }
        if grid1.lod_origin[1] != grid2.lod_origin[1] {
            continue;
        }
        if grid1.lod_origin[2] != grid2.lod_origin[2] {
            continue;
        }
        //
        while R_StitchPatches(grid1num, j, world_data) {
            numstitches += 1;
        }
    }
    numstitches
}

// --- R3 wave 4 ---------------------------------------------------------

/// Raven `R_StitchAllPatches`.
///
/// PORT-NOTE: same "no state channel" / plain-`world_data` shape as its
/// `R_TryStitchingPatch`/`R_StitchPatches` siblings above — it walks
/// `worldData.surfaces` as a `&mut [Surface]` slice (DEC-43.1);
/// `worldData.numsurfaces` is `world_data.len()`. `numstitches` is
/// transcribed as a local accumulator even though its only oracle consumer,
/// the trailing `Com_Printf`, is commented out in the oracle itself — dead by
/// construction, not a Rust-side drop (porting-rules §2).
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp:1281-1307`
pub fn R_StitchAllPatches(world_data: &mut [Surface]) {
    let mut numstitches = 0i32;
    loop {
        let mut stitched = false;
        for i in 0..world_data.len() {
            //
            // if this surface is not a grid
            let Some(grid) = surface_grid_mut(&mut world_data[i]) else {
                continue;
            };
            //
            if grid.lod_stitched != 0 {
                continue;
            }
            //
            grid.lod_stitched = 1;
            stitched = true;
            //
            numstitches += R_TryStitchingPatch(i, world_data);
        }
        if !stitched {
            break;
        }
    }
    let _ = numstitches;
    //	Com_Printf ("stitched %d LoD cracks\n", numstitches );
}

// --- R3 wave 8 ---------------------------------------------------------

/// Raven `ShaderForShaderNum`.
///
/// PORT-NOTE: `world` is the oracle's `world_t &worldData` reference
/// parameter (a specific BSP instance the caller passes, not necessarily
/// `tr.world`) — read-only here, matching the oracle body (no field of
/// `worldData` is mutated). The `R_FindShader` bundle
/// (`qs`/`frame`/`assets`/`view`/`cvars`/`models`/`img_state`/
/// `sky_view`/`sky`) is threaded verbatim in the shape wave 7 landed it
/// (`tr_shader.rs::R_FindShader`) — this fn has no state of its own beyond
/// what that call needs.
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp:318-351`
#[allow(clippy::too_many_arguments)]
fn ShaderForShaderNum(
    qs: &mut QSharedScratch,
    world_load: &mut WorldLoadState,
    assets: &mut RenderAssets,
    view: &mut EngineHostView,
    cvars: &RendererCvars,
    models: &RenderModels,
    img_state: &mut TrImageState,
    sky_view: &mut viewParms_t,
    shader_num: i32,
    lightmap_num: &[i32],
    lightmap_styles: &[u8],
    vertex_styles: &[u8],
    world: &WorldAsset,
) -> ShaderHandle {
    let mut styles = lightmap_styles;
    let mut lightmap_num = lightmap_num;

    let shader_num = LittleLong(shader_num);
    if shader_num < 0 || shader_num >= world.shaders.len() as i32 {
        com_error(
            errorParm_t::ERR_DROP,
            format!("ShaderForShaderNum: bad num {}", shader_num),
        );
    }
    let dsh = &world.shaders[shader_num as usize];

    if lightmap_num[0] == LIGHTMAP_BY_VERTEX {
        styles = vertex_styles;
    }

    if view.common.cvar(cvars.r_vertexLight).integer != 0 {
        lightmap_num = &lightmapsVertex;
        styles = vertex_styles;
    }

    let shader = R_FindShader(
        &dsh.shader,
        lightmap_num,
        styles,
        true,
        qs,
        world_load,
        assets,
        view,
        cvars,
        models,
        img_state,
        sky_view,
    );

    // if the shader had errors, just use default shader
    let is_default_shader = assets
        .shaders
        .get(shader)
        .map(|sh| sh.default_shader)
        .unwrap_or(false);
    if is_default_shader {
        return ShaderHandle::slot_zero(); // tr.defaultShader
    }

    shader
}

/// Raven `R_LoadFogs`.
///
/// PORT-NOTE: `world` is the `world_t &worldData` output parameter this loader
/// fills (same treatment as `R_LoadShaders`/`R_LoadPlanes`/
/// `R_LoadNodesAndLeafs`/`R_LoadSubmodels` above); `assets` is threaded to
/// read `tr.world` for the nightvision-fog-slot copy (`## State ownership`
/// row `tr` registries → `RenderAssets`) and, with the rest of the
/// `R_FindShader` bundle, for the per-fog shader lookup.
/// `worldData.numfogs` collapses to
/// `world.fogs.len()` (this file's established collection-length-is-the-count
/// pattern); the `Hunk_Alloc`/pointer-walk (`out = worldData.fogs + 1;
/// out++`) collapses to indexing `world.fogs[i + 1]` directly (same
/// `Hunk_Alloc`/`Com_Mem*` collapse as `R_LoadVisibility` above).
///
/// The oracle's fn-local `int lightmaps[MAXLIGHTMAPS] = { LIGHTMAP_NONE };`
/// (`:1674`) is a C *partial* initializer — first element `LIGHTMAP_NONE`,
/// the rest zero — so it is `{-1, 0, 0, 0}`, not the all-`-1` `lightmapsNone`
/// table; transcribed as the literal, same as `ParseFlare`'s `{
/// LIGHTMAP_BY_VERTEX }`.
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp:1662-1804`
#[allow(clippy::too_many_arguments)]
fn R_LoadFogs(
    qs: &mut QSharedScratch,
    world_load: &mut WorldLoadState,
    assets: &mut RenderAssets,
    view: &mut EngineHostView,
    cvars: &RendererCvars,
    models: &RenderModels,
    img_state: &mut TrImageState,
    sky_view: &mut viewParms_t,
    ctx: &BspLoadContext,
    l: &lump_t,
    brushes_lump: &lump_t,
    sides_lump: &lump_t,
    world: &mut WorldAsset,
    index: i32,
) {
    let lightmaps = [LIGHTMAP_NONE, 0, 0, 0];
    let entry_size = size_of::<dfog_t>();
    if (l.filelen as usize) % entry_size != 0 {
        com_error(
            errorParm_t::ERR_DROP,
            format!("LoadMap: funny lump size in {}", world.name),
        );
    }
    let count = l.filelen as usize / entry_size;

    // create fog structures for them
    let mut fogs: Vec<Fog> = vec![Fog::default(); count + 1];
    world.global_fog = -1;

    // Copy the global fog from the main world into the bsp instance
    if index != 0 {
        if let Some(w) = &assets.world {
            if w.global_fog != -1 {
                // Use the nightvision fog slot
                //
                // Raven writes `fogs[numfogs]` into a `count + 1`-entry
                // allocation — one past its end (heap overrun); the push
                // reaches the same logical index legally (porting-rules §19).
                let copied = w.fogs[w.global_fog as usize];
                world.global_fog = fogs.len() as i32;
                fogs.push(copied);
            }
        }
    }

    world.fogs = fogs;

    if count == 0 {
        return;
    }

    let brush_entry_size = size_of::<dbrush_t>();
    if (brushes_lump.filelen as usize) % brush_entry_size != 0 {
        com_error(
            errorParm_t::ERR_DROP,
            format!("LoadMap: funny lump size in {}", world.name),
        );
    }
    let brushes_count = brushes_lump.filelen as usize / brush_entry_size;
    let brushes_base = brushes_lump.fileofs as usize;

    let side_entry_size = size_of::<dbrushside_t>();
    if (sides_lump.filelen as usize) % side_entry_size != 0 {
        com_error(
            errorParm_t::ERR_DROP,
            format!("LoadMap: funny lump size in {}", world.name),
        );
    }
    let sides_count = sides_lump.filelen as usize / side_entry_size;
    let sides_base = sides_lump.fileofs as usize;

    let fog_base = l.fileofs as usize;

    for i in 0..count {
        let rec = &ctx.file_base[fog_base + i * entry_size..fog_base + (i + 1) * entry_size];
        // dfog_t: `shader[MAX_QPATH]` @0, `brushNum` @64, `visibleSide` @68.
        let brush_num = LittleLong(i32::from_le_bytes(rec[64..68].try_into().unwrap()));
        let visible_side = LittleLong(i32::from_le_bytes(rec[68..72].try_into().unwrap()));

        let mut out = Fog {
            original_brush_number: brush_num,
            ..Fog::default()
        };

        let first_side: i32;

        if out.original_brush_number == -1 {
            out.bounds[0] = [MIN_WORLD_COORD; 3];
            out.bounds[1] = [MAX_WORLD_COORD; 3];
            first_side = -1;
            world.global_fog = (i + 1) as i32;
        } else {
            if out.original_brush_number as u32 >= brushes_count as u32 {
                com_error(
                    errorParm_t::ERR_DROP,
                    "fog brushNumber out of range".to_string(),
                );
            }
            let brush_rec = &ctx.file_base[brushes_base
                + out.original_brush_number as usize * brush_entry_size
                ..brushes_base + (out.original_brush_number as usize + 1) * brush_entry_size];
            // dbrush_t::firstSide @0.
            first_side = LittleLong(i32::from_le_bytes(brush_rec[0..4].try_into().unwrap()));

            // C's `(unsigned)firstSide > sidesCount - 6` computes `sidesCount
            // - 6` in `int` (wrapping negative for a tiny sides lump) before
            // the unsigned comparison — reproduced with the same width/order.
            let threshold = (sides_count as i32 - 6) as u32;
            if (first_side as u32) > threshold {
                com_error(
                    errorParm_t::ERR_DROP,
                    "fog brush sideNumber out of range".to_string(),
                );
            }

            // brushes are always sorted with the axial sides first
            for n in 0..6i32 {
                let side_num = (first_side + n) as usize;
                let side_rec = &ctx.file_base[sides_base + side_num * side_entry_size
                    ..sides_base + (side_num + 1) * side_entry_size];
                // dbrushside_t::planeNum @0.
                let plane_num = LittleLong(i32::from_le_bytes(side_rec[0..4].try_into().unwrap()));
                let dist = world.planes[plane_num as usize].dist;
                let axis = (n / 2) as usize;
                if n % 2 == 0 {
                    out.bounds[0][axis] = -dist;
                } else {
                    out.bounds[1][axis] = dist;
                }
            }
        }

        // get information from the shader for fog parameters
        let shader = R_FindShader(
            &latin1_name(&rec[0..MAX_QPATH]),
            &lightmaps,
            &stylesDefault,
            true,
            qs,
            world_load,
            assets,
            view,
            cvars,
            models,
            img_state,
            sky_view,
        );

        match assets.shaders.get(shader).and_then(|sh| sh.fog_parms) {
            None => {
                //bad shader!!
                // (Raven's companion `assert(shader->fogParms)` is a
                // debug-only diagnostic with no ported counterpart.)
                out.parms.color[0] = 1.0;
                out.parms.color[1] = 0.0;
                out.parms.color[2] = 0.0;
                // Raven's `out->parms.color[3] = 0.0f` writes one past the
                // 3-float `color`, clobbering `depthForOpaque` — which the
                // next line re-sets, so the OOB write is dropped (§19).
                out.parms.depth_for_opaque = 250.0;
            }
            Some(parms) => out.parms = parms,
        }

        out.color_int = ColorBytes4(
            out.parms.color[0] * world_load.identity_light,
            out.parms.color[1] * world_load.identity_light,
            out.parms.color[2] * world_load.identity_light,
            1.0,
        );
        let d = if out.parms.depth_for_opaque < 1.0 {
            1.0
        } else {
            out.parms.depth_for_opaque
        };
        out.tc_scale = 1.0 / (d * 8.0);

        // set the gradient vector
        if visible_side == -1 {
            //rww - we need to set this to qtrue for global fog as well
            out.has_surface = true;
        } else {
            out.has_surface = true;
            // `firstSide + sideNum` cast to `usize`: if this ever goes
            // negative (C reads `sides[-1]`, UB) the cast wraps to a huge
            // index and the slice indexing below panics — the one defined
            // behavior for otherwise-nonsensical input (porting-rules §19).
            let side_num = (first_side + visible_side) as usize;
            let side_rec = &ctx.file_base[sides_base + side_num * side_entry_size
                ..sides_base + (side_num + 1) * side_entry_size];
            // dbrushside_t::planeNum @0.
            let plane_num = LittleLong(i32::from_le_bytes(side_rec[0..4].try_into().unwrap()));
            let plane = &world.planes[plane_num as usize];
            let mut dir = [0.0f32; 3];
            _VectorSubtract(vec3_origin, plane.normal, &mut dir);
            out.surface[0..3].copy_from_slice(&dir);
            out.surface[3] = -plane.dist;
        }

        world.fogs[i + 1] = out;
    }
}

// --- R3 wave 9 -----------------------------------------------------------
//
// The four `Parse*` fns below (`ParseFace`/`ParseMesh`/`ParseTriSurf`/
// `ParseFlare`) each fill in Raven's `msurface_t *surf` out-parameter. Per
// the out-params→returns dictionary entry, each returns an owned `Parsed*`
// triple (`fog_index`, `shader`, and the fn's own owned surface payload)
// instead — `ParseFace`/`ParseTriSurf` realize the tier-2 transition audit's
// Group 1 `srfSurfaceFace_t`/`srfTriangles_t` rows as new owned types
// (`SurfaceFace`/`SurfaceTriangles`) since this wave is their owning
// construction site; `ParseMesh` reuses the already-owned `GridMesh`
// (`tr_curve.rs`, wave 2); `ParseFlare` reuses the existing tier-2
// `srfFlare_t` as-is, since it carries zero raw-pointer/`c_char`/`qboolean`
// fields (no audit row — genuinely pointer-free already). `R_LoadSurfaces`
// (the caller these fns serve) assembles each triple into one `Surface` on
// `WorldAsset::surfaces` — the DEC-43 carrier, defined below.
//
// CROSS-FILE ESCALATION (found by grepping the workspace, not assumed):
// `tr_surface.rs`'s already-declared `RB_SurfaceFace(surf: &srfSurfaceFace_t,
// ...)`/`RB_SurfaceTriangles(srf: &srfTriangles_t, ...)` (both `todo!()`
// stubs, signatures only) still expect the tier-2 raw-pointer shapes this
// wave deliberately does not build. Reconciling `SurfaceFace`/
// `SurfaceTriangles` with those signatures — or re-pointing them at the new
// owned shapes — is out of this packet's `tr_bsp.rs`-only scope; flagged for
// whichever wave finishes `RB_SurfaceFace`/`RB_SurfaceTriangles` or designs
// the real `WorldAsset::surfaces` arena.

/// Owned per-point payload of `SurfaceFace::points` — Raven's packed
/// `points[VERTEXSIZE]` float row (`oracle/codemp/renderer/tr_local.h:730`)
/// reshaped to a typed struct instead. The interior-safety law disfavors
/// reconstructing the C byte-reinterpret trick that row packs (`(byte
/// *)&cv->points[i][VERTEX_COLOR+k]`), and the `VERTEX_LM`/`VERTEX_COLOR`
/// column-offset `#define`s that trick needs are genuinely absent from this
/// packet — neither in `## FILE-SCOPE CONSTANTS` nor `ParseFace`'s own
/// oracle slice (they live in `tr_local.h`, a different TU) — never-guess
/// rule (porting-rules §A2). This struct carries the identical information
/// losslessly without needing those offsets at all.
#[derive(Clone, Copy)]
pub struct FaceVertex {
    pub xyz: Vec3,
    pub st: [f32; 2],
    pub lightmap: [[f32; 2]; MAXLIGHTMAPS],
    pub color: [[u8; 4]; MAXLIGHTMAPS],
}

/// Owned replacement for Raven `srfSurfaceFace_t` (tier-2 transition audit,
/// Group 1 `srfSurfaceFace_t` row; this wave's `ParseFace` is the owning
/// construction site). The flexible trailing arrays (`points[numPoints]`,
/// then `numIndices` ints at `ofsIndices`) collapse to two owned `Vec`s — no
/// `Hunk_Alloc` flexible-array-member trick needed once the surface is owned
/// rather than hunk-backed (same collapse as this file's
/// `R_LoadVisibility`/`R_LoadNodesAndLeafs` precedent). Field spelled
/// `indices` (not `indexes`) to match Raven's own `numIndices`/`ofsIndices`
/// naming for this specific type.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:799-812`
#[derive(Clone)]
pub struct SurfaceFace {
    pub plane: cplane_t,
    // W2-F4 moved `dlightBits` out of the surface. The loaded world is
    // immutable after load, so `R_DlightFace` writes
    // `WorldWalkScratch::surf_dlight_bits` at the owning surface's index.
    pub points: Vec<FaceVertex>,
    pub indices: Vec<i32>,
}

/// Owned replacement for Raven `srfTriangles_t` (tier-2 transition audit,
/// Group 1 `srfTriangles_t` row; this wave's `ParseTriSurf` is the owning
/// construction site). `verts`/`indexes` collapse to owned `Vec`s — no
/// `Hunk_Alloc` single-block-with-two-trailing-arrays trick needed. Field
/// spelled `indexes` (not `indices`) to match Raven's own
/// `numIndexes`/`indexes` naming for this specific type (Raven itself is
/// inconsistent between `srfSurfaceFace_t` and `srfTriangles_t`).
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:818-836`
// `Clone` added by DEC-43.4, once `drawVert_t` gained its own derive.
#[derive(Clone)]
pub struct SurfaceTriangles {
    // W2-F4 moved `dlightBits` out of the surface. See `SurfaceFace` above for
    // the same note.
    /// `bounds[2]` — culling information.
    pub bounds: [Vec3; 2],
    pub verts: Vec<drawVert_t>,
    pub indexes: Vec<i32>,
}

/// The owned form of Raven's `msurface_t.data` tagged union — the
/// `surfaceType_t *` whose leading discriminant selects one of the `srf*_t`
/// structs behind it. One variant per arm of `R_LoadSurfaces`' `switch`
/// (`MST_PLANAR`/`MST_PATCH`/`MST_TRIANGLE_SOUP`/`MST_FLARE`), plus `Skip`
/// for `ParseMesh`'s nodraw early return (`surf->data = &skipData`).
///
/// DEC-43.2: there is deliberately **no** `Terrain` variant. `SF_TERRAIN`
/// appears nowhere in `tr_bsp.cpp`/`tr_world.cpp` — terrain enters the draw
/// list as the engine-global `&tr.landScape`
/// (`oracle/codemp/renderer/tr_terrain.cpp:1005`), never as a BSP surface —
/// so a variant here would invent state the oracle lacks (porting-rules §A2).
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:656-678`
#[derive(Clone)]
pub enum SurfaceData {
    /// `&skipData` — Raven's shared `static surfaceType_t skipData = SF_SKIP`
    /// nodraw tag (three-kind rule kind 1: a const, not cross-frame state).
    Skip,
    /// `srfSurfaceFace_t` (`SF_FACE`).
    Face(SurfaceFace),
    /// `srfGridMesh_t` (`SF_GRID`).
    Grid(GridMesh),
    /// `srfTriangles_t` (`SF_TRIANGLES`).
    Triangles(SurfaceTriangles),
    /// `srfFlare_t` (`SF_FLARE`).
    Flare(srfFlare_t),
}

/// Owned replacement for Raven `msurface_t` — one renderable BSP surface,
/// element of [`WorldAsset::surfaces`] (DEC-43.1). `shader: shader_s *`
/// becomes a `ShaderHandle` and the `data: surfaceType_t *` tagged-union
/// pointer becomes the owned [`SurfaceData`]; the array stays flat and in
/// lump order, so `WorldAsset::mark_surfaces` and `BModel`'s
/// `first_surface`/`num_surfaces` range address it with the oracle's own
/// surface indices (`worldData.numsurfaces` is `surfaces.len()`).
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:872-878`
#[derive(Clone)]
pub struct Surface {
    // W2-F4 moved `viewCount` out of the surface. The loaded world is
    // immutable after load, so `R_AddWorldSurface` stamps
    // `WorldWalkScratch::surf_view_count` at this surface's flat index.
    // Source: `oracle/codemp/renderer/tr_local.h:874`
    /// `shader`.
    pub shader: ShaderHandle,
    /// `fogIndex`.
    pub fog_index: i32,
    /// `data` — Raven: any of `srf*_t`.
    pub data: SurfaceData,
}

/// `(srfGridMesh_t *)worldData.surfaces[i].data` behind the oracle's own
/// `surfaceType != SF_GRID` guard — `None` is that guard's `continue`.
fn surface_grid(surf: &Surface) -> Option<&GridMesh> {
    match &surf.data {
        SurfaceData::Grid(grid) => Some(grid),
        _ => None,
    }
}

/// Mutable twin of [`surface_grid`].
fn surface_grid_mut(surf: &mut Surface) -> Option<&mut GridMesh> {
    match &mut surf.data {
        SurfaceData::Grid(grid) => Some(grid),
        _ => None,
    }
}

/// The owned triple `ParseFace` computes in place of writing through Raven's
/// `msurface_t *surf` out-parameter — see this section's header comment for
/// why. `fog_index`/`shader` are the two fields every `Parse*` sibling in
/// this wave computes identically; `face` is `ParseFace`'s own payload.
pub struct ParsedFace {
    pub fog_index: i32,
    pub shader: ShaderHandle,
    pub face: SurfaceFace,
}

/// The two `msurface_t.data` shapes `ParseMesh` can produce — `Skip` for the
/// nodraw-surface early return (`surf->data = &skipData`), `Grid` otherwise.
/// Raven's fn-scope `static surfaceType_t skipData = SF_SKIP;` is a shared,
/// never-mutated constant tag (three-kind rule kind 1 — a const, not
/// cross-frame state) — represented here as the `Skip` unit variant rather
/// than a `static`.
pub enum MeshSurfaceData {
    Skip,
    Grid(GridMesh),
}

/// The owned triple `ParseMesh` computes — see this section's header comment
/// for the out-param → return-value translation.
pub struct ParsedMesh {
    pub fog_index: i32,
    pub shader: ShaderHandle,
    pub data: MeshSurfaceData,
}

/// The owned triple `ParseFlare` computes — see this section's header
/// comment for the out-param → return-value translation. `flare` reuses the
/// existing tier-2 `srfFlare_t` as-is (it carries zero raw-pointer/`c_char`/
/// `qboolean` fields, so constructing it needs no `unsafe` and does not
/// extend the tier-2 pattern).
pub struct ParsedFlare {
    pub fog_index: i32,
    pub shader: ShaderHandle,
    pub flare: srfFlare_t,
}

/// Raven's file-scope `#define LIGHTMAP_SIZE 128` — in-packet FILE-SCOPE
/// CONSTANT.
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp:167`
const LIGHTMAP_SIZE: usize = 128;

/// Raven `R_LoadLightmaps`.
///
/// PORT-NOTE: the oracle's `&worldData == &s_worldData` pointer-identity
/// check (deciding whether to force-reset `tr.numLightmaps` to 0 before an
/// empty-lump early return) has no Rust equivalent while `world` would be
/// held as a distinct `&mut WorldAsset` alongside `assets.world` — translated
/// to an explicit `is_main_world` flag the caller supplies (DEC-37 A13.3-style
/// naming, not a state channel). `tr.numLightmaps` itself collapses to
/// `assets.lightmaps.len()` (this file's established collection-length-is-
/// the-count pattern): the pre-clear only has an observable effect on the
/// `filelen == 0` early-return leg (the unconditional `tr.numLightmaps = len /
/// (...)` a few lines later in the oracle overwrites it otherwise), so it is
/// applied only there.
///
/// `tr.numLightmaps` *is* read elsewhere — `tr_shader.cpp:1320` (the
/// `$lightmap` stage guard) and `:3443` (`R_FindShader`'s "use fullbright
/// vertex lighting if the bsp has no lightmaps" substitution), which this
/// port spells as `assets.lightmaps.len()` at `tr_shader.rs:3780`/`:4867`.
/// On the `r_vertexLight` early return (`tr_bsp.cpp:191-196`) the oracle
/// leaves `numLightmaps` at the nonzero on-disk count while never filling
/// `tr.lightmaps[]`, so both guards see "lightmaps exist" and hand out the
/// stale/NULL `tr.lightmaps[index]`; this port's `len()` stays 0, so both
/// guards take their no-lightmap arm (`lightmapsVertex` / `tr.whiteImage`) —
/// the defined behavior in place of Raven's stale-pointer read, and the
/// behavior `r_vertexLight` asks for anyway (porting-rules §19).
///
/// The `R_ColorShiftLightingBytes(&buf_p[j*3], &image[j*4])` call passes a
/// 3-byte source to the 4-byte-in/4-byte-out overload, so its 4th "alpha"
/// input byte is really the next pixel's red channel in the oracle (or, for
/// the lightmap lump's very last pixel, one byte past the lump). Since that
/// call's 4th *output* byte is unconditionally overwritten immediately after
/// (`image[j*4+3] = 255`), the 4th input byte's value is behaviorally inert —
/// this port passes a `0` placeholder instead of reproducing the read, both
/// because it cannot affect the result and because the tail-of-lump case
/// would read past `ctx.file_base`'s real `Vec` length outright (unlike C's
/// larger backing allocation, which reads adjacent-but-irrelevant memory
/// rather than panicking).
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp:168-247`
#[allow(clippy::too_many_arguments)]
pub fn R_LoadLightmaps(
    world_load: &WorldLoadState,
    assets: &mut RenderAssets,
    view: &mut EngineHostView,
    cvars: &RendererCvars,
    models: &RenderModels,
    img_state: &mut TrImageState,
    ctx: &BspLoadContext,
    l: &lump_t,
    ps_map_name: &str,
    is_main_world: bool,
) {
    if is_main_world {
        assets.lightmaps.clear();
    }

    let filelen = l.filelen;
    if filelen == 0 {
        return;
    }

    // we are about to upload textures
    R_SyncRenderThread(assets, view.common, cvars);

    // create all the lightmaps
    let num_lightmaps = filelen as usize / (LIGHTMAP_SIZE * LIGHTMAP_SIZE * 3);

    // if we are in r_vertexLight mode, we don't need the lightmaps at all
    if view.common.cvar(cvars.r_vertexLight).integer != 0 {
        // Leaves `assets.lightmaps` empty where the oracle leaves
        // `tr.numLightmaps` nonzero over an unfilled `tr.lightmaps[]` — see
        // this fn's doc comment (porting-rules §19).
        return;
    }

    let s_map_name = COM_StripExtension(ps_map_name); // will already by MAX_QPATH legal, so no length check

    let mut max_intensity = 0.0f32;
    let mut sum_intensity = 0.0f64;

    let buf_base = l.fileofs as usize;
    let mut lightmaps: Vec<ImageHandle> = Vec::new();
    for i in 0..num_lightmaps {
        // expand the 24 bit on-disk to 32 bit
        let lm_base = buf_base + i * LIGHTMAP_SIZE * LIGHTMAP_SIZE * 3;
        let mut image = vec![0u8; LIGHTMAP_SIZE * LIGHTMAP_SIZE * 4];

        if view.common.cvar(cvars.r_lightmap).integer == 2 {
            // color code by intensity as development tool (FIXME: check range)
            for j in 0..LIGHTMAP_SIZE * LIGHTMAP_SIZE {
                let src = lm_base + j * 3;
                let r = ctx.file_base[src] as f32;
                let g = ctx.file_base[src + 1] as f32;
                let b = ctx.file_base[src + 2] as f32;

                let mut intensity = 0.33f32 * r + 0.685f32 * g + 0.063f32 * b;
                if intensity > 255.0 {
                    intensity = 1.0;
                } else {
                    intensity /= 255.0;
                }

                if intensity > max_intensity {
                    max_intensity = intensity;
                }

                let out = HSVtoRGB(intensity, 1.00, 0.50);

                image[j * 4] = (out[0] * 255.0) as u8;
                image[j * 4 + 1] = (out[1] * 255.0) as u8;
                image[j * 4 + 2] = (out[2] * 255.0) as u8;
                image[j * 4 + 3] = 255;

                sum_intensity += intensity as f64;
            }
        } else {
            for j in 0..LIGHTMAP_SIZE * LIGHTMAP_SIZE {
                let src = lm_base + j * 3;
                // See this fn's doc comment: the 4th input byte is
                // behaviorally inert (its output slot is overwritten below).
                let color_in = [
                    ctx.file_base[src],
                    ctx.file_base[src + 1],
                    ctx.file_base[src + 2],
                    0,
                ];
                let out = R_ColorShiftLightingBytes(world_load, color_in);
                image[j * 4] = out[0];
                image[j * 4 + 1] = out[1];
                image[j * 4 + 2] = out[2];
                image[j * 4 + 3] = 255;
            }
        }

        let allow_tc = view.common.cvar(cvars.r_ext_compressed_lightmaps).integer != 0;
        let handle = R_CreateImage(
            view,
            cvars,
            assets,
            models,
            img_state,
            &format!("*{}/lightmap{}", s_map_name, i),
            &image,
            LIGHTMAP_SIZE as i32,
            LIGHTMAP_SIZE as i32,
            GL_RGBA,
            false,
            false,
            allow_tc,
            GL_CLAMP,
            false,
        );
        lightmaps.push(handle);
    }

    assets.lightmaps = lightmaps;

    if view.common.cvar(cvars.r_lightmap).integer == 2 {
        let _ = sum_intensity; // computed faithfully, never read by the oracle either
        com_printf(
            view.common,
            &format!(
                "Brightest lightmap value: {}\n",
                (max_intensity * 255.0) as i32
            ),
        );
    }
}

/// Raven `R_GetShaderByNum`.
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp:300-311`
#[allow(clippy::too_many_arguments)]
pub fn R_GetShaderByNum(
    qs: &mut QSharedScratch,
    world_load: &mut WorldLoadState,
    assets: &mut RenderAssets,
    view: &mut EngineHostView,
    cvars: &RendererCvars,
    models: &RenderModels,
    img_state: &mut TrImageState,
    sky_view: &mut viewParms_t,
    shader_num: i32,
    world: &WorldAsset,
) -> i32 {
    if shader_num < 0 || shader_num >= world.shaders.len() as i32 {
        com_printf(
            view.common,
            &format!("Warning: Bad index for R_GetShaderByNum - {}", shader_num),
        );
        return 0;
    }
    RE_RegisterShader(
        &world.shaders[shader_num as usize].shader,
        qs,
        world_load,
        assets,
        view,
        cvars,
        models,
        img_state,
        sky_view,
    )
}

/// Raven `ParseFace`.
///
/// PORT-NOTE: see this section's header comment for the out-param →
/// return-value translation.
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp:358-435`
#[allow(clippy::too_many_arguments)]
pub fn ParseFace(
    qs: &mut QSharedScratch,
    world_load: &mut WorldLoadState,
    assets: &mut RenderAssets,
    view: &mut EngineHostView,
    cvars: &RendererCvars,
    models: &RenderModels,
    img_state: &mut TrImageState,
    sky_view: &mut viewParms_t,
    ds: &dsurface_t,
    verts: &[mapVert_t],
    indexes: &[i32],
    world: &WorldAsset,
    index: i32,
) -> ParsedFace {
    // get fog volume
    let mut fog_index = LittleLong(ds.fogNum) + 1;
    if index != 0 && fog_index == 0 {
        if let Some(w) = &assets.world {
            if w.global_fog != -1 {
                fog_index = world.global_fog;
            }
        }
    }

    let mut lightmap_num = [0i32; MAXLIGHTMAPS];
    for i in 0..MAXLIGHTMAPS {
        lightmap_num[i] = LittleLong(ds.lightmapNum[i]);
    }

    // get shader value
    let mut shader = ShaderForShaderNum(
        qs,
        world_load,
        assets,
        view,
        cvars,
        models,
        img_state,
        sky_view,
        ds.shaderNum,
        &lightmap_num,
        &ds.lightmapStyles,
        &ds.vertexStyles,
        world,
    );
    if view.common.cvar(cvars.r_singleShader).integer != 0 {
        let is_sky = assets
            .shaders
            .get(shader)
            .map(|sh| sh.sky.is_some())
            .unwrap_or(false);
        if !is_sky {
            shader = ShaderHandle::slot_zero(); // tr.defaultShader
        }
    }

    let mut num_points = LittleLong(ds.numVerts);
    if num_points > MAX_FACE_POINTS {
        // S_COLOR_YELLOW ("^3"), `mp_qshared::shared::q_color::S_COLOR_YELLOW`.
        com_printf(
            view.common,
            &format!("^3WARNING: MAX_FACE_POINTS exceeded: {num_points}\n"),
        );
        num_points = MAX_FACE_POINTS;
        shader = ShaderHandle::slot_zero(); // tr.defaultShader
    }

    let num_indexes = LittleLong(ds.numIndexes);

    let first_vert = LittleLong(ds.firstVert) as usize;
    let mut points: Vec<FaceVertex> = Vec::new();
    for i in 0..num_points as usize {
        let v = &verts[first_vert + i];
        let xyz = [
            LittleFloat(v.xyz[0]),
            LittleFloat(v.xyz[1]),
            LittleFloat(v.xyz[2]),
        ];
        let st = [LittleFloat(v.st[0]), LittleFloat(v.st[1])];
        let mut lightmap = [[0.0f32; 2]; MAXLIGHTMAPS];
        for k in 0..MAXLIGHTMAPS {
            lightmap[k] = [LittleFloat(v.lightmap[k][0]), LittleFloat(v.lightmap[k][1])];
        }
        let mut color = [[0u8; 4]; MAXLIGHTMAPS];
        for k in 0..MAXLIGHTMAPS {
            color[k] = R_ColorShiftLightingBytes(world_load, v.color[k]);
        }
        points.push(FaceVertex {
            xyz,
            st,
            lightmap,
            color,
        });
    }

    let first_index = LittleLong(ds.firstIndex) as usize;
    let mut indices: Vec<i32> = Vec::new();
    for i in 0..num_indexes as usize {
        indices.push(LittleLong(indexes[first_index + i]));
    }

    // take the plane information from the lightmap vector
    let mut normal = [0.0f32; 3];
    for i in 0..3 {
        normal[i] = LittleFloat(ds.lightmapVecs[2][i]);
    }
    // At `num_points == 0` Raven reads the uninitialized hunk slot
    // `cv->points[0]`; the indexing panics here instead (porting-rules §19).
    let dist = _DotProduct(points[0].xyz, normal);
    // `SetPlaneSignbits` inlined per this file's `R_LoadPlanes` precedent —
    // avoids the raw-pointer `*mut cplane_t` call surface for an equivalent
    // three-line computation.
    let mut signbits = 0u8;
    for (i, &n) in normal.iter().enumerate() {
        if n < 0.0 {
            signbits |= 1 << i;
        }
    }
    let plane_type = PlaneTypeForNormal(normal);

    ParsedFace {
        fog_index,
        shader,
        face: SurfaceFace {
            plane: cplane_t {
                normal,
                dist,
                r#type: plane_type as u8,
                signbits,
                pad: [0, 0],
            },
            points,
            indices,
        },
    }
}

/// Raven `ParseMesh`.
///
/// PORT-NOTE: see this section's header comment for the out-param →
/// return-value translation, and `MeshSurfaceData`'s doc comment for the
/// `skipData` fn-scope static's three-kind classification.
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp:443-516`
#[allow(clippy::too_many_arguments)]
pub fn ParseMesh(
    qs: &mut QSharedScratch,
    world_load: &mut WorldLoadState,
    assets: &mut RenderAssets,
    view: &mut EngineHostView,
    cvars: &RendererCvars,
    models: &RenderModels,
    img_state: &mut TrImageState,
    sky_view: &mut viewParms_t,
    ds: &dsurface_t,
    verts: &[mapVert_t],
    world: &WorldAsset,
    index: i32,
) -> ParsedMesh {
    let mut lightmap_num = [0i32; MAXLIGHTMAPS];
    for i in 0..MAXLIGHTMAPS {
        lightmap_num[i] = LittleLong(ds.lightmapNum[i]);
    }

    // get fog volume
    let mut fog_index = LittleLong(ds.fogNum) + 1;
    if index != 0 && fog_index == 0 {
        if let Some(w) = &assets.world {
            if w.global_fog != -1 {
                fog_index = world.global_fog;
            }
        }
    }

    // get shader value
    let mut shader = ShaderForShaderNum(
        qs,
        world_load,
        assets,
        view,
        cvars,
        models,
        img_state,
        sky_view,
        ds.shaderNum,
        &lightmap_num,
        &ds.lightmapStyles,
        &ds.vertexStyles,
        world,
    );
    if view.common.cvar(cvars.r_singleShader).integer != 0 {
        let is_sky = assets
            .shaders
            .get(shader)
            .map(|sh| sh.sky.is_some())
            .unwrap_or(false);
        if !is_sky {
            shader = ShaderHandle::slot_zero(); // tr.defaultShader
        }
    }

    // we may have a nodraw surface, because they might still need to
    // be around for movement clipping
    let shader_num = LittleLong(ds.shaderNum);
    if world.shaders[shader_num as usize].surface_flags & SURF_NODRAW != 0 {
        return ParsedMesh {
            fog_index,
            shader,
            data: MeshSurfaceData::Skip,
        };
    }

    let width = LittleLong(ds.patchWidth);
    let height = LittleLong(ds.patchHeight);

    let first_vert = LittleLong(ds.firstVert) as usize;
    let num_points = (width * height) as usize;
    let mut points: Vec<drawVert_t> = Vec::new();
    for i in 0..num_points {
        let v = &verts[first_vert + i];
        let mut color = [[0u8; 4]; MAXLIGHTMAPS];
        for k in 0..MAXLIGHTMAPS {
            color[k] = R_ColorShiftLightingBytes(world_load, v.color[k]);
        }
        let mut lightmap = [[0.0f32; 2]; MAXLIGHTMAPS];
        for k in 0..MAXLIGHTMAPS {
            lightmap[k] = [LittleFloat(v.lightmap[k][0]), LittleFloat(v.lightmap[k][1])];
        }
        points.push(drawVert_t {
            xyz: [
                LittleFloat(v.xyz[0]),
                LittleFloat(v.xyz[1]),
                LittleFloat(v.xyz[2]),
            ],
            st: [LittleFloat(v.st[0]), LittleFloat(v.st[1])],
            lightmap,
            normal: [
                LittleFloat(v.normal[0]),
                LittleFloat(v.normal[1]),
                LittleFloat(v.normal[2]),
            ],
            color,
        });
    }

    // pre-tesseleate
    let mut grid =
        R_SubdividePatchToGrid(width as usize, height as usize, &points, view.common, cvars);

    // copy the level of detail origin, which is the center
    // of the group of all curves that must subdivide the same
    // to avoid cracking
    let mut bounds0 = [0.0f32; 3];
    let mut bounds1 = [0.0f32; 3];
    for i in 0..3 {
        bounds0[i] = LittleFloat(ds.lightmapVecs[0][i]);
        bounds1[i] = LittleFloat(ds.lightmapVecs[1][i]);
    }
    _VectorAdd(bounds0, bounds1, &mut bounds1);
    _VectorScale(bounds1, 0.5, &mut grid.lod_origin);
    let mut tmp_vec = [0.0f32; 3];
    _VectorSubtract(bounds0, grid.lod_origin, &mut tmp_vec);
    grid.lod_radius = VectorLength(tmp_vec);

    ParsedMesh {
        fog_index,
        shader,
        data: MeshSurfaceData::Grid(grid),
    }
}

/// Raven `ParseTriSurf`.
///
/// PORT-NOTE: see this section's header comment for the out-param →
/// return-value translation.
///
/// `Com_Error` → `com_error` (`R2-D11`): receiverless, panics, never a
/// `Result` — matches the packet's threading digest note for this fn.
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp:523-592`
#[allow(clippy::too_many_arguments)]
pub fn ParseTriSurf(
    qs: &mut QSharedScratch,
    world_load: &mut WorldLoadState,
    assets: &mut RenderAssets,
    view: &mut EngineHostView,
    cvars: &RendererCvars,
    models: &RenderModels,
    img_state: &mut TrImageState,
    sky_view: &mut viewParms_t,
    ds: &dsurface_t,
    verts: &[mapVert_t],
    indexes: &[i32],
    world: &WorldAsset,
    index: i32,
) -> ParsedTriSurf {
    // get fog volume
    let mut fog_index = LittleLong(ds.fogNum) + 1;
    if index != 0 && fog_index == 0 {
        if let Some(w) = &assets.world {
            if w.global_fog != -1 {
                fog_index = world.global_fog;
            }
        }
    }

    // get shader
    let mut shader = ShaderForShaderNum(
        qs,
        world_load,
        assets,
        view,
        cvars,
        models,
        img_state,
        sky_view,
        ds.shaderNum,
        &lightmapsVertex,
        &ds.lightmapStyles,
        &ds.vertexStyles,
        world,
    );
    if view.common.cvar(cvars.r_singleShader).integer != 0 {
        let is_sky = assets
            .shaders
            .get(shader)
            .map(|sh| sh.sky.is_some())
            .unwrap_or(false);
        if !is_sky {
            shader = ShaderHandle::slot_zero(); // tr.defaultShader
        }
    }

    let num_verts = LittleLong(ds.numVerts);
    let num_indexes = LittleLong(ds.numIndexes);

    if num_verts >= SHADER_MAX_VERTEXES as i32 {
        let name = assets
            .shaders
            .get(shader)
            .map(|sh| sh.name.clone())
            .unwrap_or_default();
        com_error(
            errorParm_t::ERR_DROP,
            format!(
                "ParseTriSurf: verts > MAX ({num_verts} > {SHADER_MAX_VERTEXES}) on misc_model {name}"
            ),
        );
    }
    if num_indexes >= SHADER_MAX_INDEXES as i32 {
        let name = assets
            .shaders
            .get(shader)
            .map(|sh| sh.name.clone())
            .unwrap_or_default();
        com_error(
            errorParm_t::ERR_DROP,
            format!(
                "ParseTriSurf: indices > MAX ({num_indexes} > {SHADER_MAX_INDEXES}) on misc_model {name}"
            ),
        );
    }

    // copy vertexes
    let mut mins = [0.0f32; 3];
    let mut maxs = [0.0f32; 3];
    ClearBoundsMP(&mut mins, &mut maxs);
    let first_vert = LittleLong(ds.firstVert) as usize;
    let mut tri_verts: Vec<drawVert_t> = Vec::new();
    for i in 0..num_verts as usize {
        let v = &verts[first_vert + i];
        let xyz = [
            LittleFloat(v.xyz[0]),
            LittleFloat(v.xyz[1]),
            LittleFloat(v.xyz[2]),
        ];
        let normal = [
            LittleFloat(v.normal[0]),
            LittleFloat(v.normal[1]),
            LittleFloat(v.normal[2]),
        ];
        AddPointToBounds(xyz, &mut mins, &mut maxs);
        let st = [LittleFloat(v.st[0]), LittleFloat(v.st[1])];
        let mut lightmap = [[0.0f32; 2]; MAXLIGHTMAPS];
        for k in 0..MAXLIGHTMAPS {
            lightmap[k] = [LittleFloat(v.lightmap[k][0]), LittleFloat(v.lightmap[k][1])];
        }
        let mut color = [[0u8; 4]; MAXLIGHTMAPS];
        for k in 0..MAXLIGHTMAPS {
            color[k] = R_ColorShiftLightingBytes(world_load, v.color[k]);
        }
        tri_verts.push(drawVert_t {
            xyz,
            st,
            lightmap,
            normal,
            color,
        });
    }

    // copy indexes
    let first_index = LittleLong(ds.firstIndex) as usize;
    let mut tri_indexes: Vec<i32> = Vec::new();
    for i in 0..num_indexes as usize {
        let idx = LittleLong(indexes[first_index + i]);
        if idx < 0 || idx >= num_verts {
            com_error(
                errorParm_t::ERR_DROP,
                "Bad index in triangle surface".to_string(),
            );
        }
        tri_indexes.push(idx);
    }

    ParsedTriSurf {
        fog_index,
        shader,
        tri: SurfaceTriangles {
            bounds: [mins, maxs],
            verts: tri_verts,
            indexes: tri_indexes,
        },
    }
}

/// The owned triple `ParseTriSurf` computes — see this section's header
/// comment for the out-param → return-value translation.
pub struct ParsedTriSurf {
    pub fog_index: i32,
    pub shader: ShaderHandle,
    pub tri: SurfaceTriangles,
}

/// Raven `ParseFlare`.
///
/// PORT-NOTE: see this section's header comment for the out-param →
/// return-value translation. `lightmaps[MAXLIGHTMAPS] = { LIGHTMAP_BY_VERTEX
/// }` is a C partial initializer (first element set, the rest
/// zero-initialized) — transcribed directly, unlike `ParseTriSurf`'s
/// file-scope `lightmapsVertex` table, since this literal is given verbatim
/// in this fn's own oracle slice (not a deferred, elsewhere-defined table).
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp:599-627`
#[allow(clippy::too_many_arguments)]
pub fn ParseFlare(
    qs: &mut QSharedScratch,
    world_load: &mut WorldLoadState,
    assets: &mut RenderAssets,
    view: &mut EngineHostView,
    cvars: &RendererCvars,
    models: &RenderModels,
    img_state: &mut TrImageState,
    sky_view: &mut viewParms_t,
    ds: &dsurface_t,
    world: &WorldAsset,
    index: i32,
) -> ParsedFlare {
    // get fog volume
    let mut fog_index = LittleLong(ds.fogNum) + 1;
    if index != 0 && fog_index == 0 {
        if let Some(w) = &assets.world {
            if w.global_fog != -1 {
                fog_index = world.global_fog;
            }
        }
    }

    // get shader
    let lightmaps = [LIGHTMAP_BY_VERTEX, 0, 0, 0];
    let mut shader = ShaderForShaderNum(
        qs,
        world_load,
        assets,
        view,
        cvars,
        models,
        img_state,
        sky_view,
        ds.shaderNum,
        &lightmaps,
        &ds.lightmapStyles,
        &ds.vertexStyles,
        world,
    );
    if view.common.cvar(cvars.r_singleShader).integer != 0 {
        let is_sky = assets
            .shaders
            .get(shader)
            .map(|sh| sh.sky.is_some())
            .unwrap_or(false);
        if !is_sky {
            shader = ShaderHandle::slot_zero(); // tr.defaultShader
        }
    }

    let mut origin = [0.0f32; 3];
    let mut color = [0.0f32; 3];
    let mut normal = [0.0f32; 3];
    for i in 0..3 {
        origin[i] = LittleFloat(ds.lightmapOrigin[i]);
        color[i] = LittleFloat(ds.lightmapVecs[0][i]);
        normal[i] = LittleFloat(ds.lightmapVecs[2][i]);
    }

    ParsedFlare {
        fog_index,
        shader,
        flare: srfFlare_t {
            surfaceType: surfaceType_t::SF_FLARE,
            origin,
            normal,
            color,
        },
    }
}

// --- R3 wave 10 ----------------------------------------------------------

/// Raw-byte decode of one on-disk `dsurface_t` record — same field-by-field
/// pattern as this file's `R_LoadNodesAndLeafs`/`R_LoadPlanes`/
/// `R_LoadShaders` (wave 1/0) decoders. `from_le_bytes` here is the
/// byte-level read standing in for the oracle's raw `(dsurface_t *)(fileBase
/// + ...)` cast, not a swap: the oracle's own swap calls are the
/// `LittleLong`/`LittleFloat` that `ParseFace`/`ParseMesh`/`ParseTriSurf`/
/// `ParseFlare` apply to every field they read, so no `LittleLong` is
/// wrapped around the decode (doing both would be a redundant double swap;
/// on WIN32 they are identity anyway,
/// `crates/mp/qshared/src/shared/swap.rs:100-109`).
///
/// Field layout: `oracle/codemp/qcommon/qfiles.h:538-559`
/// (`crates/mp/engine/qcommon/src/qfiles/dsurface_t.rs`'s asserted offsets).
fn decode_dsurface(rec: &[u8]) -> dsurface_t {
    let i32_at = |off: usize| i32::from_le_bytes(rec[off..off + 4].try_into().unwrap());
    let f32_at = |off: usize| f32::from_le_bytes(rec[off..off + 4].try_into().unwrap());

    let mut lightmapStyles = [0u8; MAXLIGHTMAPS];
    lightmapStyles.copy_from_slice(&rec[28..28 + MAXLIGHTMAPS]);
    let mut vertexStyles = [0u8; MAXLIGHTMAPS];
    vertexStyles.copy_from_slice(&rec[32..32 + MAXLIGHTMAPS]);

    let mut lightmapNum = [0i32; MAXLIGHTMAPS];
    let mut lightmapX = [0i32; MAXLIGHTMAPS];
    let mut lightmapY = [0i32; MAXLIGHTMAPS];
    for k in 0..MAXLIGHTMAPS {
        lightmapNum[k] = i32_at(36 + k * 4);
        lightmapX[k] = i32_at(52 + k * 4);
        lightmapY[k] = i32_at(68 + k * 4);
    }

    let mut lightmapOrigin = [0.0f32; 3];
    for k in 0..3 {
        lightmapOrigin[k] = f32_at(92 + k * 4);
    }
    let mut lightmapVecs = [[0.0f32; 3]; 3];
    for v in 0..3 {
        for k in 0..3 {
            lightmapVecs[v][k] = f32_at(104 + v * 12 + k * 4);
        }
    }

    dsurface_t {
        shaderNum: i32_at(0),
        fogNum: i32_at(4),
        surfaceType: i32_at(8),
        firstVert: i32_at(12),
        numVerts: i32_at(16),
        firstIndex: i32_at(20),
        numIndexes: i32_at(24),
        lightmapStyles,
        vertexStyles,
        lightmapNum,
        lightmapX,
        lightmapY,
        lightmapWidth: i32_at(84),
        lightmapHeight: i32_at(88),
        lightmapOrigin,
        lightmapVecs,
        patchWidth: i32_at(140),
        patchHeight: i32_at(144),
    }
}

/// Raw-byte decode of one on-disk `mapVert_t` record — see
/// [`decode_dsurface`]'s doc comment for the shared decode rationale.
///
/// Field layout: `oracle/codemp/qcommon/qfiles.h:506-512`
/// (`crates/mp/engine/qcommon/src/qfiles/map_vert_t.rs`'s asserted offsets).
fn decode_map_vert(rec: &[u8]) -> mapVert_t {
    let f32_at = |off: usize| f32::from_le_bytes(rec[off..off + 4].try_into().unwrap());

    let mut xyz = [0.0f32; 3];
    for k in 0..3 {
        xyz[k] = f32_at(k * 4);
    }
    let mut st = [0.0f32; 2];
    for k in 0..2 {
        st[k] = f32_at(12 + k * 4);
    }
    let mut lightmap = [[0.0f32; 2]; MAXLIGHTMAPS];
    for m in 0..MAXLIGHTMAPS {
        for k in 0..2 {
            lightmap[m][k] = f32_at(20 + m * 8 + k * 4);
        }
    }
    let mut normal = [0.0f32; 3];
    for k in 0..3 {
        normal[k] = f32_at(52 + k * 4);
    }
    let mut color = [[0u8; 4]; MAXLIGHTMAPS];
    for m in 0..MAXLIGHTMAPS {
        color[m].copy_from_slice(&rec[64 + m * 4..68 + m * 4]);
    }

    mapVert_t {
        xyz,
        st,
        lightmap,
        normal,
        color,
    }
}

/// Raven `R_LoadSurfaces`.
///
/// PORT-NOTE: `dsurface_t`/`mapVert_t`/the index array are decoded from
/// `fileBase` bytes up front (`decode_dsurface`/`decode_map_vert`, same
/// pattern as `R_LoadNodesAndLeafs`/`R_LoadPlanes` above); each surface
/// record is then dispatched by `surfaceType` exactly like the oracle
/// `switch`, and the `Parsed*` triple its `Parse*` callee returns (the
/// out-param → return-value translation of Raven's `msurface_t *surf`, wave
/// 9) is assembled into one `Surface` and pushed onto `world.surfaces` —
/// so surface order is lump order and the flat index space is the oracle's
/// (DEC-43.1). `viewCount` is `0` for every fresh surface, matching the
/// oracle's zeroed `Hunk_Alloc(count * sizeof(msurface_t), h_low)`;
/// `worldData.numsurfaces = count` is `world.surfaces.len()`.
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp:1346-1412`
#[allow(clippy::too_many_arguments)]
pub fn R_LoadSurfaces(
    qs: &mut QSharedScratch,
    world_load: &mut WorldLoadState,
    assets: &mut RenderAssets,
    view: &mut EngineHostView,
    cvars: &RendererCvars,
    models: &RenderModels,
    img_state: &mut TrImageState,
    sky_view: &mut viewParms_t,
    ctx: &BspLoadContext,
    surfs: &lump_t,
    verts_lump: &lump_t,
    index_lump: &lump_t,
    world: &mut WorldAsset,
    index: i32,
) {
    let surf_entry_size = size_of::<dsurface_t>();
    if (surfs.filelen as usize) % surf_entry_size != 0 {
        com_error(
            errorParm_t::ERR_DROP,
            format!("LoadMap: funny lump size in {}", world.name),
        );
    }
    let count = surfs.filelen as usize / surf_entry_size;
    let surf_base = surfs.fileofs as usize;

    let vert_entry_size = size_of::<mapVert_t>();
    if (verts_lump.filelen as usize) % vert_entry_size != 0 {
        com_error(
            errorParm_t::ERR_DROP,
            format!("LoadMap: funny lump size in {}", world.name),
        );
    }
    let vert_count = verts_lump.filelen as usize / vert_entry_size;
    let vert_base = verts_lump.fileofs as usize;
    let mut dv: Vec<mapVert_t> = Vec::with_capacity(vert_count);
    for i in 0..vert_count {
        let rec =
            &ctx.file_base[vert_base + i * vert_entry_size..vert_base + (i + 1) * vert_entry_size];
        dv.push(decode_map_vert(rec));
    }

    let index_entry_size = size_of::<i32>();
    if (index_lump.filelen as usize) % index_entry_size != 0 {
        com_error(
            errorParm_t::ERR_DROP,
            format!("LoadMap: funny lump size in {}", world.name),
        );
    }
    let index_count = index_lump.filelen as usize / index_entry_size;
    let index_base = index_lump.fileofs as usize;
    let mut indexes: Vec<i32> = Vec::with_capacity(index_count);
    for i in 0..index_count {
        // Byte decode only — the oracle's `indexes = (int *)(fileBase +
        // indexLump->fileofs)` is an unswapped cast, and `ParseFace`/
        // `ParseTriSurf` are where its `LittleLong` lands (same single-swap
        // structure as `decode_dsurface`/`decode_map_vert`).
        indexes.push(i32::from_le_bytes(
            ctx.file_base
                [index_base + i * index_entry_size..index_base + (i + 1) * index_entry_size]
                .try_into()
                .unwrap(),
        ));
    }

    // `out = Hunk_Alloc(count * sizeof(msurface_t)); worldData.surfaces =
    // out; worldData.numsurfaces = count;` — the owned flat `Vec` grown one
    // `Surface` per lump record below (DEC-43.1).
    // Source: oracle/codemp/renderer/tr_bsp.cpp:1373-1376
    let mut surfaces: Vec<Surface> = Vec::with_capacity(count);

    for i in 0..count {
        let rec =
            &ctx.file_base[surf_base + i * surf_entry_size..surf_base + (i + 1) * surf_entry_size];
        let ds = decode_dsurface(rec);
        let surface_type = LittleLong(ds.surfaceType);

        if surface_type == mapSurfaceType_t::MST_PATCH as i32 {
            let parsed = ParseMesh(
                qs, world_load, assets, view, cvars, models, img_state, sky_view, &ds, &dv,
                world, index,
            );
            surfaces.push(Surface {
                shader: parsed.shader,
                fog_index: parsed.fog_index,
                data: match parsed.data {
                    MeshSurfaceData::Skip => SurfaceData::Skip,
                    MeshSurfaceData::Grid(grid) => SurfaceData::Grid(grid),
                },
            });
        } else if surface_type == mapSurfaceType_t::MST_TRIANGLE_SOUP as i32 {
            let parsed = ParseTriSurf(
                qs, world_load, assets, view, cvars, models, img_state, sky_view, &ds, &dv,
                &indexes, world, index,
            );
            surfaces.push(Surface {
                shader: parsed.shader,
                fog_index: parsed.fog_index,
                data: SurfaceData::Triangles(parsed.tri),
            });
        } else if surface_type == mapSurfaceType_t::MST_PLANAR as i32 {
            let parsed = ParseFace(
                qs, world_load, assets, view, cvars, models, img_state, sky_view, &ds, &dv,
                &indexes, world, index,
            );
            surfaces.push(Surface {
                shader: parsed.shader,
                fog_index: parsed.fog_index,
                data: SurfaceData::Face(parsed.face),
            });
        } else if surface_type == mapSurfaceType_t::MST_FLARE as i32 {
            let parsed = ParseFlare(
                qs, world_load, assets, view, cvars, models, img_state, sky_view, &ds, world,
                index,
            );
            surfaces.push(Surface {
                shader: parsed.shader,
                fog_index: parsed.fog_index,
                data: SurfaceData::Flare(parsed.flare),
            });
        } else {
            com_error(errorParm_t::ERR_DROP, "Bad surfaceType".to_string());
        }
    }

    world.surfaces = surfaces;

    R_StitchAllPatches(&mut world.surfaces);
    R_FixSharedVertexLodError(&mut world.surfaces);
    R_MovePatchSurfacesToHunk(&mut world.surfaces);

    // Com_Printf("...loaded %d faces, %i meshes, %i trisurfs, %i
    // flares\n", ...) — commented out in the oracle; nothing to port.
}

/// Raven `R_LoadEntities`.
///
/// PORT-NOTE: `COM_ParseExt(&p, qtrue)` → `COM_Parse(cursor, true)`, the
/// merged `COM_Parse`/`COM_ParseExt` alias this file's `R_GetEntityToken`
/// (wave 0) already established
/// (`crates/mp/qshared/src/shared/q_string.rs:247-251`) — the packet's "NOT
/// RESOLVED" note is superseded by that in-file precedent.
/// `Hunk_Alloc`+`strcpy` for `w->entityString` collapses to this file's
/// `latin1_name` helper (NUL-truncated, Latin-1 discipline); the oracle's
/// separate `p` walking pointer and `w->entityString` copy are two views of
/// the identical bytes, so this port decodes once and walks the same owned
/// `text` for both. `Q_strncmp(keyname, s, strlen(s))` (case-sensitive
/// prefix) → `str::starts_with`; `Q_stricmp(keyname, ...)` (case-insensitive
/// equality) → `str::eq_ignore_ascii_case` — both already this crate's
/// established `char*`→`&str` translations (`tr_shader.rs`).
/// `strchr(value, ';')` + in-place NUL write → `str::find(';')` +
/// `&str` split. `sscanf(value, "%f"/"%f %f %f", ...)` →
/// `native_string::sscanf::sscanf_f32s` (the crate's canonical native-libc
/// `sscanf` scanner — never `.parse()` per the marker law), which already
/// implements sscanf's "leave unmatched destinations untouched" semantics by
/// seeding `out` with the pre-call value.
///
/// `tr.sunAmbient`/`tr.distanceCull` per R2 `## State ownership`: frontend
/// scratch → `FrameState::sun_ambient` (this wave's field-merge addition,
/// see the top-of-file note); sim-readable → `RenderAssets::distance_cull`.
///
/// Panics via `R_RemapShader`'s loud stub (this file's `tr_shader.rs`,
/// wave 9) if either a `vertexremapshader`/`remapshader` key is present in
/// the worldspawn.
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp:1878-1971`
#[allow(clippy::too_many_arguments)]
pub fn R_LoadEntities(
    qs: &mut QSharedScratch,
    world_load: &mut WorldLoadState,
    assets: &mut RenderAssets,
    view: &mut EngineHostView,
    cvars: &RendererCvars,
    models: &RenderModels,
    img_state: &mut TrImageState,
    sky_view: &mut viewParms_t,
    ctx: &BspLoadContext,
    l: &lump_t,
    world: &mut WorldAsset,
) {
    let mut ambient = 1.0f32;

    world.light_grid_size = [64.0, 64.0, 128.0];

    world_load.sun_ambient = [1.0, 1.0, 1.0];
    assets.distance_cull = 6000.0; // DEFAULT_DISTANCE_CULL

    let base = l.fileofs as usize;
    let filelen = l.filelen as usize;
    let raw = &ctx.file_base[base..base + filelen];
    // store for reference by the cgame
    let text = latin1_name(raw);
    world.entity_string = text.clone();
    world.entity_parse_point = 0;

    let mut cursor: &str = &text;
    let (token, rest) = COM_Parse(cursor, true);
    cursor = rest;
    if token.is_empty() || !token.starts_with('{') {
        return;
    }

    // only parse the world spawn
    loop {
        // parse key
        let (token, rest) = COM_Parse(cursor, true);
        cursor = rest;
        if token.is_empty() || token.starts_with('}') {
            break;
        }
        let keyname = token;

        // parse value
        let (token, rest) = COM_Parse(cursor, true);
        cursor = rest;
        if token.is_empty() || token.starts_with('}') {
            break;
        }
        let value = token;

        // check for remapping of shaders for vertex lighting
        if keyname.starts_with("vertexremapshader") {
            let Some(pos) = value.find(';') else {
                com_printf(
                    view.common,
                    &format!(
                        "^3WARNING: no semi colon in vertexshaderremap '{}'\n",
                        value
                    ),
                );
                break;
            };
            let shader_name = &value[..pos];
            let new_shader_name = &value[pos + 1..];
            if view.common.cvar(cvars.r_vertexLight).integer != 0 {
                R_RemapShader(
                    shader_name,
                    new_shader_name,
                    Some("0"),
                    qs,
                    world_load,
                    assets,
                    view,
                    cvars,
                    models,
                    img_state,
                    sky_view,
                );
            }
            continue;
        }
        // check for remapping of shaders
        if keyname.starts_with("remapshader") {
            let Some(pos) = value.find(';') else {
                com_printf(
                    view.common,
                    &format!("^3WARNING: no semi colon in shaderremap '{}'\n", value),
                );
                break;
            };
            let shader_name = &value[..pos];
            let new_shader_name = &value[pos + 1..];
            R_RemapShader(
                shader_name,
                new_shader_name,
                Some("0"),
                qs,
                world_load,
                assets,
                view,
                cvars,
                models,
                img_state,
                sky_view,
            );
            continue;
        }
        if keyname.eq_ignore_ascii_case("distanceCull") {
            let mut out = [assets.distance_cull];
            sscanf_f32s(&value, &mut out);
            assets.distance_cull = out[0];
            continue;
        }
        // check for a different grid size
        if keyname.eq_ignore_ascii_case("gridsize") {
            let mut out = world.light_grid_size;
            sscanf_f32s(&value, &mut out);
            world.light_grid_size = out;
            continue;
        }
        // find the optional world ambient for arioche
        if keyname.eq_ignore_ascii_case("_color") {
            let mut out = world_load.sun_ambient;
            sscanf_f32s(&value, &mut out);
            world_load.sun_ambient = out;
            continue;
        }
        if keyname.eq_ignore_ascii_case("ambient") {
            let mut out = [ambient];
            sscanf_f32s(&value, &mut out);
            ambient = out[0];
            continue;
        }
    }
    // both default to 1 so no harm if not present.
    let sun_ambient = world_load.sun_ambient;
    _VectorScale(sun_ambient, ambient, &mut world_load.sun_ambient);
}

// --- R3 wave 11 --------------------------------------------------------

/// Raven `RE_LoadWorldMap_Actual` — the BSP-file loader entry point: reads
/// the file (cached image or disk), swaps/decodes the `dheader_t` lump
/// directory, and drives every lower-wave `R_Load*` loader in the oracle's
/// own call order.
///
/// STATE HOMES (this packet's row): `tr` is SPLIT — `worldMapLoaded`
/// (session flag) → `RenderAssets::world_map_loaded` (this wave's field-
/// merge addition, top-of-file note); `sunDirection` (frontend scratch) →
/// `FrameState::sun_direction` (already real); `tr.world` (registry) →
/// `RenderAssets::world`. `com_RMG` is engine-owned, reached through
/// `view.common.com_RMG`/`.cvar()` (engine-fork ruling 2). `fileBase`
/// becomes this fn's locally-owned `BspLoadContext` (per that type's own
/// doc comment — "owned by whichever caller drives `R_LoadWorld`'s call
/// tree"; this fn *is* that caller).
///
/// Three deliberate departures from a literal transcription, each cited at
/// its own site below (`skyboxportal = 0` was a fourth until campaign #41
/// batch 1 gave it a home on `FrameState::skyboxportal`; it is now written
/// literally):
/// - `c_gridVerts = 0` — DEFERRED, no R2 carrier and zero consumers found
///   anywhere in the ported crate (grep-verified).
/// - the cached-disk-image read (`gpvCachedMapDiskImage`) — a loud
///   `todo!()`: the engine's `CollisionWorld::gpvCachedMapDiskImage` carries
///   no companion length field to read it without a banned unbounded-length
///   unsafe deref. The branch is reachable on the client track (`CM_LoadMap`,
///   `cm_load.rs:1284-1290`, frees the buffer only under
///   `Sys_LowPhysicalMemory() || com_dedicated`); the retention seam is
///   unwired, not dead.
/// - `worldData.dataSize` (the `Hunk_Alloc` before/after difference) —
///   dropped, matching this file's own `R_LoadVisibility`/
///   `R_MovePatchSurfacesToHunk` Hunk_Alloc-collapse precedent; `WorldAsset`
///   carries no `data_size` field for the same reason.
///
/// `COM_SkipPath(worldData.name)` has no idiomatic `&str` port in the
/// workspace (only a `*mut c_char` version, wrong crate and barred by the
/// interior-safety law) — this file's sibling `tr_font.rs` (`:497-501`)
/// already resolved the identical dependency by spelling the two-line scan
/// out over `&str`; matched here for consistency.
///
/// `tr.world = &worldData` aliases the *same* object `R_RMGInit` then
/// mutates in place; the owned-clone model (`assets.world =
/// Some(world.clone())`) cannot alias that way, so the `com_RMG` branch
/// copies `R_RMGInit`'s `assets.world` mutations back into the `world`
/// out-param afterward — see the PORT-NOTE at that call site.
///
/// Panics via `R_RemapShader`'s loud stub (`tr_shader.rs`, wave 9) through
/// `R_LoadEntities`, on the same conditions that callee's own doc comment
/// describes.
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp:2003-2110`
#[allow(clippy::too_many_arguments)]
pub fn RE_LoadWorldMap_Actual(
    qs: &mut QSharedScratch,
    world_load: &mut WorldLoadState,
    assets: &mut RenderAssets,
    view: &mut EngineHostView,
    cvars: &RendererCvars,
    models: &mut RenderModels,
    img_state: &mut TrImageState,
    sky_view: &mut viewParms_t,
    world_effects: &mut WorldEffectsState,
    name: &str,
    world: &mut WorldAsset,
    index: i32,
) {
    if assets.world_map_loaded && index == 0 {
        com_error(
            errorParm_t::ERR_DROP,
            "ERROR: attempted to redundantly load world map\n".to_string(),
        );
    }

    if index == 0 {
        // The `skyboxportal = 0` reset lives in `RE_LoadWorldMap`, this fn's
        // only `index == 0` caller. See that fn for why (W2-F3).
        // Source: oracle/codemp/renderer/tr_bsp.cpp:2016

        // set default sun direction to be used if it isn't
        // overridden by a shader
        world_load.sun_direction = [0.45, 0.3, 0.9];

        VectorNormalize(&mut world_load.sun_direction);

        assets.world_map_loaded = true;

        // clear tr.world so if the level fails to load, the next
        // try will not look at the partially loaded version
        assets.world = None;
    }

    // check for cached disk file from the server first...
    //
    // Raven reads the retained block in place and frees it after the load
    // (`tr_bsp.cpp:2101-2104`); the take copies and frees now, so this file
    // never touches the raw pointer.
    let buffer: Vec<u8> = if let Some(cached) = CM_TakeCachedMapDiskImage(view) {
        cached
    } else {
        // still needs loading...
        match FS_ReadFileVec(view, name) {
            Some(buf) => buf,
            None => com_error(
                errorParm_t::ERR_DROP,
                format!("RE_LoadWorldMap: {name} not found"),
            ),
        }
    };

    *world = WorldAsset::default();

    // Q_strncpyz(worldData.name, name, sizeof(worldData.name))
    let mut world_name = name.to_string();
    if world_name.len() > MAX_QPATH - 1 {
        world_name.truncate(MAX_QPATH - 1);
    }
    world.name = world_name;

    // Q_strncpyz(worldData.baseName, COM_SkipPath(worldData.name),
    // sizeof(worldData.name)); COM_StripExtension(baseName, baseName);
    //
    // PORT-NOTE: `COM_SkipPath` has no idiomatic `&str`-based port anywhere
    // in the workspace reachable from this crate (only a raw `*mut c_char`
    // version exists, `crates/mp/game/src/q_shared.rs:82` — wrong crate, and
    // its `c_char` shape is barred by the interior-safety law); this file's
    // sibling `tr_font.rs` (`:497-501`) already reached the same conclusion
    // and spelled the two-line scan out over `&str` — matched here for
    // consistency rather than re-escalating an already-resolved dependency.
    let skip_path = match world.name.rsplit_once('/') {
        Some((_, tail)) => tail,
        None => world.name.as_str(),
    };
    let mut base_name = skip_path.to_string();
    if base_name.len() > MAX_QPATH - 1 {
        base_name.truncate(MAX_QPATH - 1);
    }
    world.base_name = COM_StripExtension(&base_name);

    // DEFERRED: c_gridVerts = 0 — file-scope static with no R2-assigned
    // carrier (DEC-37 A13.3). Zero consumers anywhere in the ported crate
    // (grep-verified: `tr_curve.rs`'s already-landed grid-subdivision code,
    // the only plausible incrementer, never references it) — a kind-3
    // cross-call diagnostic counter this wave has no evidence for beyond its
    // own reset site, so naming a carrier here would be speculation, not a
    // licensed home.
    // Source: oracle/codemp/renderer/tr_bsp.cpp:2057

    // PORT-NOTE: `startMarker`/`Hunk_Alloc(0, h_low)` before/after the load,
    // and the resulting `worldData.dataSize = ... - startMarker`, measure
    // hunk-memory consumption for a diagnostic stat — the same `Hunk_Alloc`
    // collapse this file's `R_LoadVisibility` PORT-NOTE already established
    // (no idiomatic-interior counterpart; owned `Vec`s have no hunk to
    // measure against). `WorldAsset` carries no `data_size` field for the
    // same reason `R_MovePatchSurfacesToHunk` is a no-op here — dropped,
    // matching precedent, not silently: this note is that citation.

    // A file too short to hold a `dheader_t` panics on these slice reads
    // where Raven reads past its allocation and usually reaches the
    // "wrong version number" `ERR_DROP` — the defined behavior (§19).
    let version = LittleLong(i32::from_le_bytes(buffer[4..8].try_into().unwrap()));
    if version != BSP_VERSION {
        com_error(
            errorParm_t::ERR_DROP,
            format!(
                "RE_LoadWorldMap: {name} has wrong version number ({version} should be {BSP_VERSION})"
            ),
        );
    }

    // swap all the lumps
    //
    // PORT-NOTE: the oracle's in-place `((int *)header)[i] = LittleLong(...)`
    // byte-swap sweep over the whole `dheader_t` collapses into decoding
    // each `lump_t` field with `LittleLong` at read time below — this file's
    // established per-field decode idiom (`R_LoadShaders`/`R_LoadPlanes`/...
    // above), rather than a separate whole-header pass. `ident` is never
    // read by this fn (only `CM_LoadMap` checks it), so it is not decoded.
    let mut lumps: Vec<lump_t> = Vec::with_capacity(HEADER_LUMPS);
    for i in 0..HEADER_LUMPS {
        let base = 8 + i * 8;
        let fileofs = LittleLong(i32::from_le_bytes(
            buffer[base..base + 4].try_into().unwrap(),
        ));
        let filelen = LittleLong(i32::from_le_bytes(
            buffer[base + 4..base + 8].try_into().unwrap(),
        ));
        lumps.push(lump_t { fileofs, filelen });
    }

    let ctx = BspLoadContext { file_base: buffer };

    // load into heap
    R_LoadShaders(&ctx, &lumps[LUMP_SHADERS], world);
    R_LoadLightmaps(
        world_load,
        assets,
        view,
        cvars,
        models,
        img_state,
        &ctx,
        &lumps[LUMP_LIGHTMAPS],
        name,
        index == 0,
    );
    R_LoadPlanes(&ctx, &lumps[LUMP_PLANES], world);
    R_LoadFogs(
        qs,
        world_load,
        assets,
        view,
        cvars,
        models,
        img_state,
        sky_view,
        &ctx,
        &lumps[LUMP_FOGS],
        &lumps[LUMP_BRUSHES],
        &lumps[LUMP_BRUSHSIDES],
        world,
        index,
    );
    R_LoadSurfaces(
        qs,
        world_load,
        assets,
        view,
        cvars,
        models,
        img_state,
        sky_view,
        &ctx,
        &lumps[LUMP_SURFACES],
        &lumps[LUMP_DRAWVERTS],
        &lumps[LUMP_DRAWINDEXES],
        world,
        index,
    );
    R_LoadMarksurfaces(&ctx, &lumps[LUMP_LEAFSURFACES], world);
    R_LoadNodesAndLeafs(&ctx, &lumps[LUMP_NODES], &lumps[LUMP_LEAFS], world);
    R_LoadSubmodels(&ctx, &lumps[LUMP_MODELS], world, models, index);
    R_LoadVisibility(
        &ctx,
        assets.external_vis_data.as_ref(),
        &lumps[LUMP_VISIBILITY],
        world,
    );

    if index == 0 {
        R_LoadEntities(
            qs,
            world_load,
            assets,
            view,
            cvars,
            models,
            img_state,
            sky_view,
            &ctx,
            &lumps[LUMP_ENTITIES],
            world,
        );
        R_LoadLightGrid(&ctx, world_load, &lumps[LUMP_LIGHTGRID], world);
        R_LoadLightGridArray(view.common, &ctx, &lumps[LUMP_LIGHTARRAY], world);

        // only set tr.world now that we know the entire level has loaded properly
        assets.world = Some(Arc::new(world.clone()));
        // The previous map's sub-BSP instances belong to a world that is gone.
        // Raven's `tr.bspModels` array survives a map change too, but `tr.numBSPModels` alone bounds every read of
        // it, so a stale entry is unreachable there. The flat surface index space this port builds sums over the
        // whole `Vec`, so a stale world would sit inside it for the rest of the session.
        // Source: `oracle/codemp/renderer/tr_local.h:1399-1400`
        assets.bsp_models.clear();

        if let Some(h) = view.common.com_RMG {
            if view.common.cvar(h).integer != 0 {
                R_RMGInit(
                    qs,
                    world_load,
                    assets,
                    view,
                    cvars,
                    models,
                    img_state,
                    sky_view,
                    world_effects,
                );
                // PORT-NOTE: the oracle's `tr.world = &worldData` (just
                // above) makes `R_RMGInit`'s light-grid mutations visible
                // through both names — they are the same object. The
                // owned-clone model (`assets.world = Some(world.clone())`)
                // cannot alias that way, so this sync-back copies
                // `R_RMGInit`'s `assets.world` mutations back into the
                // `world` out-param — the least-surprising translation of
                // the oracle's pointer aliasing under the interior-safety
                // law's no-raw-pointers rule, applied only where the
                // aliasing is actually exercised (this branch).
                if let Some(updated) = &assets.world {
                    *world = (**updated).clone();
                }
            }
        }
    }

    // PORT-NOTE: the cached-disk-image free/null-out pair
    // (`if (gpvCachedMapDiskImage) { Z_Free(...); gpvCachedMapDiskImage =
    // NULL; } else FS_FreeFile(buffer);`) collapses to nothing here:
    // `FS_ReadFileVec` (called above) already performs its own
    // `FS_FreeFile` internally on every non-cached load, and the cached leg
    // is an unconditional loud stub above that never reaches this point —
    // so by construction this fn only ever completes having taken the
    // `FS_FreeFile` leg, which is already done.
}

// --- R3 wave 12 --------------------------------------------------------

/// Raven `RE_LoadWorldMap` — the public engine-seam entry point:
/// brackets `RE_LoadWorldMap_Actual` with the cached-map-diskimage
/// in-progress flag, then hands off. Raven's `s_worldData` (the file-scope
/// `world_t` storage `RE_LoadWorldMap_Actual` fills by out-reference) is
/// this fn's own locally-owned `WorldAsset` — it has no cross-call lifetime
/// of its own in the oracle beyond this one load, and `RE_LoadWorldMap_Actual`
/// already copies the finished result into `RenderAssets::world` itself
/// (`index == 0` leg).
///
/// `gbUsingCachedMapDataRightNow` is not a renderer-owned global (this
/// packet's STATE HOMES row) — it is `CollisionWorld::gbUsingCachedMapDataRightNow`
/// (`crates/mp/engine/qcommon/src/collision_world.rs:178`), already written by
/// this exact bracket pattern at `CM_LoadMap`
/// (`crates/mp/engine/qcommon/src/cm_load.rs:1307-1320`); reached here through
/// `view.cm`, never a new renderer field.
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp:2116-2123`
#[allow(clippy::too_many_arguments)]
pub fn RE_LoadWorldMap(
    qs: &mut QSharedScratch,
    world_load: &mut WorldLoadState,
    scene: &mut SceneState,
    assets: &mut RenderAssets,
    view: &mut EngineHostView,
    cvars: &RendererCvars,
    models: &mut RenderModels,
    img_state: &mut TrImageState,
    sky_view: &mut viewParms_t,
    world_effects: &mut WorldEffectsState,
    name: &str,
) {
    view.cm.gbUsingCachedMapDataRightNow = qtrue; // !!!!!!!!!!!!

    // skyboxportal = 0;
    // W2-F3 split the oracle's one `skyboxportal` static in two: the sim owns
    // the write side on `SceneState`, and each `RenderScene` event carries the
    // value to the render side. The reset sits here rather than inside
    // `RE_LoadWorldMap_Actual`'s `index == 0` block because this fn is that
    // block's only caller, and keeping it here spares the sub-BSP model
    // registration chain a `SceneState` it never uses.
    // Source: oracle/codemp/renderer/tr_bsp.cpp:2016
    scene.skyboxportal = 0;

    let mut world = WorldAsset::default();
    RE_LoadWorldMap_Actual(
        qs,
        world_load,
        assets,
        view,
        cvars,
        models,
        img_state,
        sky_view,
        world_effects,
        name,
        &mut world,
        0,
    );

    view.cm.gbUsingCachedMapDataRightNow = qfalse; // !!!!!!!!!!!!
}
