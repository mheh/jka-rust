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

use mp_engine_qcommon::common::{com_error, com_printf, Common};
use mp_engine_qcommon::qfiles::dleaf_t::dleaf_t;
use mp_engine_qcommon::qfiles::dmodel_t::dmodel_t;
use mp_engine_qcommon::qfiles::dnode_t::dnode_t;
use mp_engine_qcommon::qfiles::dplane_t::dplane_t;
use mp_engine_qcommon::qfiles::draw_vert_t::MAXLIGHTMAPS;
use mp_engine_qcommon::qfiles::dshader_t::dshader_t;
use mp_engine_qcommon::qfiles::lump_t::lump_t;
use mp_qshared::shared::q_math::PlaneTypeForNormal;
use mp_qshared::shared::q_string::COM_Parse;
use mp_qshared::shared::swap::{LittleFloat, LittleLong};
use mp_qshared::shared::{cplane_t, errorParm_t, MAX_QPATH};

use crate::render_state::frame_state::FrameState;
use crate::render_state::placeholders::{Vec3, WorldAsset};
use crate::tr_curve::{
    empty_grid_mesh, GridMesh, R_GridInsertColumn, R_GridInsertRow, MAX_GRID_SIZE,
};
use crate::tr_local::mgrid_t::mgrid_t;
use crate::tr_local::surface_type_t::surfaceType_t;

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
// marks both "no state channel"/"engine seam" only, and neither has a
// licensed `WorldAsset::surfaces`-style carrier yet (no `Surface` tagged-
// union home exists in R2 for `msurface_t.data`); they take a grid-mesh
// collection as a plain parameter instead (see each fn's doc comment).

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
/// `bmodel_t` row). `first_surface` is the parsed lump index — same
/// index-not-pointer treatment as `R_LoadMarksurfaces`'s `mark_surfaces`
/// above — since no `WorldAsset::surfaces` (`msurface_t` array) carrier
/// exists yet for it to address into (see `R_LoadSubmodels`'s doc comment).
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:938-942`
#[derive(Clone)]
pub struct BModel {
    pub bounds: [Vec3; 2],
    /// `firstSurface` — parsed lump index, not yet a live range into an
    /// owned surface array (no `WorldAsset::surfaces` carrier exists).
    pub first_surface: usize,
    /// `numSurfaces`.
    pub num_surfaces: i32,
}

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
pub fn R_ColorShiftLightingBytes(frame: &FrameState, color_in: [u8; 4]) -> [u8; 4] {
    // should NOT do it if overbrightBits is 0
    let mut shift = 0i32;
    if frame.overbright_bits != 0 {
        shift = 1 - frame.overbright_bits;
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
fn R_ColorShiftLightingBytesRGB(frame: &FrameState, color_in: [u8; 3]) -> [u8; 3] {
    let mut shift = 0i32;
    if frame.overbright_bits != 0 {
        shift = 1 - frame.overbright_bits;
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
pub fn RE_SetWorldVisData(frame: &mut FrameState, vis: Vec<u8>) {
    frame.external_vis_data = Some(vis);
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
fn R_LoadVisibility(ctx: &BspLoadContext, frame: &FrameState, l: &lump_t, world: &mut WorldAsset) {
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
    if let Some(external) = &frame.external_vis_data {
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

/// Borrows `world_data[a]` immutably and `world_data[b]` mutably from the
/// same slice — the split-borrow helper `R_FixSharedVertexLodError_r`'s
/// recursion needs (its `grid1`/`grid2` alias one array in the oracle,
/// interior-safety law: pointer aliasing becomes an index pair over one
/// owned slice instead of two independent raw pointers). `a == b` is
/// unreachable here because the caller applies the oracle's own
/// `lodFixed == 2` guard (`tr_bsp.cpp:689-692`) before splitting: the parent
/// frame sets `grid2->lodFixed = 2` at `tr_bsp.cpp:777` *before* recursing
/// with that same grid as `grid1` at `:778`, so the recursive frame's loop
/// skips its own `grid1` index on the guard and never reaches this call with
/// `a == b`. It panics rather than silently aliasing if that ordering is ever
/// broken (porting-rules §19 — pick one defined behavior for what is
/// otherwise nonsensical input).
fn split_grid_pair(world_data: &mut [GridMesh], a: usize, b: usize) -> (&GridMesh, &mut GridMesh) {
    if a < b {
        let (left, right) = world_data.split_at_mut(b);
        (&left[a], &mut right[0])
    } else {
        let (left, right) = world_data.split_at_mut(a);
        (&right[0], &mut left[b])
    }
}

/// Raven `R_FixSharedVertexLodError_r`.
///
/// PORT-NOTE: the packet's threading digest marks this "pure fn — no state
/// channel"; it operates on a plain `world_data: &mut [GridMesh]` rather
/// than `WorldAsset` (no licensed `WorldAsset::surfaces` carrier exists yet
/// — see the top-of-file wave-1 note). `grid1` crosses as an index into that
/// same slice (`grid1_idx`) instead of a raw pointer: the oracle's recursive
/// call re-enters with `grid2` (an element of `worldData.surfaces`) as the
/// new `grid1`, aliasing the very array the loop mutates, which Rust's
/// aliasing rules forbid via two independent references — `split_grid_pair`
/// derives both from one `&mut` borrow per iteration instead
/// (interior-safety law: pointer → index). The `grid2->surfaceType != SF_GRID`
/// guard is transcribed verbatim even though every element of `world_data`
/// is already a `GridMesh` (always `SF_GRID` by construction here) — it is
/// the oracle's own invariant check on the general `worldData.surfaces`
/// array, harmless to keep as a literal transcription (porting-rules §2).
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp:681-783`
pub fn R_FixSharedVertexLodError_r(start: usize, grid1_idx: usize, world_data: &mut [GridMesh]) {
    let mut j = start;
    while j < world_data.len() {
        let mut recurse = false;
        {
            // The two `grid2`-only guards read through plain immutable indexing
            // *before* the split borrow: the `lodFixed == 2` guard is what makes
            // `j == grid1_idx` unreachable at `split_grid_pair` (see its doc).
            // if this surface is not a grid
            if !matches!(world_data[j].surface_type, surfaceType_t::SF_GRID) {
                j += 1;
                continue;
            }
            // if the LOD errors are already fixed for this patch
            if world_data[j].lod_fixed == 2 {
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
pub fn R_MovePatchSurfacesToHunk(_world_data: &mut [GridMesh]) {
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
    frame: &FrameState,
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
            grid.ambientLight[j] = R_ColorShiftLightingBytesRGB(frame, grid.ambientLight[j]);
            grid.directLight[j] = R_ColorShiftLightingBytesRGB(frame, grid.directLight[j]);
        }

        light_grid_data.push(grid);
    }

    world.light_grid_data = Some(light_grid_data);
}

// --- R3 wave 2 ---------------------------------------------------------

/// Raven `R_FixSharedVertexLodError`.
///
/// PORT-NOTE: same "no state channel" / plain-`world_data` shape as the
/// wave-1 sibling `R_FixSharedVertexLodError_r` it drives (see the top-of-
/// file wave-1 note) — no licensed `WorldAsset::surfaces` carrier exists yet
/// for `worldData.surfaces`, so this walks a plain `world_data: &mut
/// [GridMesh]` slice instead.
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp:793-811`
pub fn R_FixSharedVertexLodError(world_data: &mut [GridMesh]) {
    for i in 0..world_data.len() {
        //
        // if this surface is not a grid
        if !matches!(world_data[i].surface_type, surfaceType_t::SF_GRID) {
            continue;
        }
        //
        if world_data[i].lod_fixed != 0 {
            continue;
        }
        //
        world_data[i].lod_fixed = 2;
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
/// value, so `grid2` is moved out of its slot with `core::mem::replace` and
/// the returned grid moved back — the owned-slice form of the oracle's
/// `worldData.surfaces[grid2num].data = (surfaceType_t *) grid2;` repoint.
///
/// PORT-NOTE: same "no state channel" / plain-`world_data` shape as its
/// `R_FixSharedVertexLodError` siblings above — no licensed
/// `WorldAsset::surfaces` carrier exists yet for `worldData.surfaces`.
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp:819-1235`
pub fn R_StitchPatches(grid1num: usize, grid2num: usize, world_data: &mut [GridMesh]) -> bool {
    let insert = stitch_scan(&world_data[grid1num], &world_data[grid2num]);
    let Some(insert) = insert else {
        return false;
    };

    // The callee bails (`None`) exactly when the grid is already
    // `MAX_GRID_SIZE` wide/tall, which `stitch_scan`'s own `>= MAX_GRID_SIZE`
    // guards already exclude; re-checked before the move so a bail can never
    // consume the grid out of its slot.
    let fits = match insert {
        StitchInsert::Column { .. } => world_data[grid2num].width < MAX_GRID_SIZE as i32,
        StitchInsert::Row { .. } => world_data[grid2num].height < MAX_GRID_SIZE as i32,
    };
    if !fits {
        return false;
    }

    let grid2 = replace(&mut world_data[grid2num], empty_grid_mesh());
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
            world_data[grid2num] = grid2;
            true
        }
        // Unreachable — see the `fits` check above.
        None => false,
    }
}

/// Raven `R_LoadSubmodels`.
///
/// PORT-NOTE: the lump-parsing half (bounds/`firstSurface`/`numSurfaces` per
/// submodel, feeding `WorldAsset::bmodels`) is fully ported below — it needs
/// no tier-2 access. The per-submodel `model_t` registration half is a
/// documented escalation (porting-rules §14 — the gap panics loudly rather
/// than silently dropping the registration): the packet's resolved call
/// surface names `R_AllocModel`/`RE_InsertModelIntoHash` as already-landed
/// (wave 0/wave 1), but the real landed shapes
/// (`RenderModels::r_alloc_model`/`re_insert_model_into_hash`,
/// `tr_model/render_models.rs:258,295`) are `pub(super)` — visible inside
/// `tr_model` only, unreachable from `tr_bsp.rs` (a sibling module, not an
/// ancestor). Even with access, wiring `model.bmodel` (`*mut bmodel_t`,
/// still tier-2 raw-pointer, `tr_local/model_s.rs`) at this submodel, or
/// `bmodel_t.firstSurface` (`*mut msurface_t`, `tr_local/bmodel_t.rs`) at a
/// surface range, needs raw-pointer construction with no safe quarantine
/// accessor — forbidden by this file's interior-safety law ("UNSAFE IS
/// BANNED IN THIS FILE"). Both are escalations for the integrate phase /
/// wave-planning, not numeric guesses.
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp:1421-1467`
fn R_LoadSubmodels(ctx: &BspLoadContext, l: &lump_t, world: &mut WorldAsset, index: i32) {
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

        // model = R_AllocModel(); assert(model != NULL);
        // model->type = MOD_BRUSH; model->bmodel = out;
        // if (index) sprintf(model->name, "*%d-%d", index, i), model->bspInstance = qtrue;
        // else sprintf(model->name, "*%d", i);
        // RE_InsertModelIntoHash(model->name, model);
        // See the ESCALATION doc comment above — genuinely unreachable from
        // this file today. Deferred to after the loop (below) so the safe
        // bounds/surface-range parsing above always completes and
        // `world.bmodels` is always fully populated before the escalation
        // fires, rather than aborting partway through the lump.
    }

    world.bmodels = bmodels;

    //TODO: Port R_LoadSubmodels model_t registration
    // Source: oracle/codemp/renderer/tr_bsp.cpp:1433-1463
    if count > 0 {
        let _ = index;
        todo!(
            "Port R_LoadSubmodels model_t registration — RenderModels::r_alloc_model/\
             re_insert_model_into_hash are pub(super) (unreachable from tr_bsp.rs), and \
             model_t.bmodel/bmodel_t.firstSurface are tier-2 raw-pointer fields with no \
             safe accessor to wire under the interior-safety law. R_LoadSubmodels as a \
             whole is unusable until that model_t registration lands — every non-empty \
             submodel lump trips this — \
             oracle/codemp/renderer/tr_bsp.cpp:1433-1463"
        );
    }
}

// --- R3 wave 3 ---------------------------------------------------------

/// Raven `R_TryStitchingPatch`.
///
/// PORT-NOTE: same "no state channel" / plain-`world_data` shape as its
/// `R_StitchPatches`/`R_FixSharedVertexLodError` siblings above — no
/// licensed `WorldAsset::surfaces` carrier exists yet for `worldData.surfaces`
/// (see the top-of-file wave-1 note), so this walks a plain `world_data: &mut
/// [GridMesh]` slice instead; `worldData.numsurfaces` is `world_data.len()`.
/// The oracle caches `grid1 = worldData.surfaces[grid1num].data` once before
/// the loop and never re-fetches it, but `lodRadius`/`lodOrigin` (the only
/// fields read off `grid1` here) are never mutated by `R_StitchPatches`, so
/// re-indexing `world_data[grid1num]` fresh each iteration is behaviorally
/// identical and avoids holding a stale reference across the mutating call
/// (interior-safety law: no raw pointers to alias here in the first place).
/// The oracle's `grid2->surfaceType != SF_GRID` guard is transcribed
/// verbatim, same as `R_FixSharedVertexLodError_r`'s identical guard, even
/// though every element of `world_data` is already a `GridMesh` here
/// (porting-rules §2).
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp:1250-1274`
pub fn R_TryStitchingPatch(grid1num: usize, world_data: &mut [GridMesh]) -> i32 {
    let mut numstitches = 0i32;
    for j in 0..world_data.len() {
        //
        // if this surface is not a grid
        if !matches!(world_data[j].surface_type, surfaceType_t::SF_GRID) {
            continue;
        }
        // grids in the same LOD group should have the exact same lod radius
        if world_data[grid1num].lod_radius != world_data[j].lod_radius {
            continue;
        }
        // grids in the same LOD group should have the exact same lod origin
        if world_data[grid1num].lod_origin[0] != world_data[j].lod_origin[0] {
            continue;
        }
        if world_data[grid1num].lod_origin[1] != world_data[j].lod_origin[1] {
            continue;
        }
        if world_data[grid1num].lod_origin[2] != world_data[j].lod_origin[2] {
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
/// `R_TryStitchingPatch`/`R_StitchPatches` siblings above — no licensed
/// `WorldAsset::surfaces` carrier exists yet for `worldData.surfaces` (see
/// the top-of-file wave-1 note), so this walks a plain `world_data: &mut
/// [GridMesh]` slice instead; `worldData.numsurfaces` is `world_data.len()`.
/// `numstitches` is transcribed as a local accumulator even though its only
/// oracle consumer, the trailing `Com_Printf`, is commented out in the
/// oracle itself — dead by construction, not a Rust-side drop
/// (porting-rules §2).
///
/// Source: `oracle/codemp/renderer/tr_bsp.cpp:1281-1307`
pub fn R_StitchAllPatches(world_data: &mut [GridMesh]) {
    let mut numstitches = 0i32;
    loop {
        let mut stitched = false;
        for i in 0..world_data.len() {
            //
            // if this surface is not a grid
            if !matches!(world_data[i].surface_type, surfaceType_t::SF_GRID) {
                continue;
            }
            //
            if world_data[i].lod_stitched != 0 {
                continue;
            }
            //
            world_data[i].lod_stitched = 1;
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
