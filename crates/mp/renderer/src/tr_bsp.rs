//! Raven `tr_bsp.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_bsp.cpp`

// Raven-named functions/types keep their original casing across this
// transcription, matching the rest of the renderer/engine crates.
#![allow(non_snake_case)]
// Wave-0 ports of Raven `static` helpers: private by fidelity, with their
// callers landing in later R3 waves.
#![allow(dead_code)]

use core::mem::size_of;

use mp_engine_qcommon::common::{com_error, com_printf, Common};
use mp_engine_qcommon::qfiles::dplane_t::dplane_t;
use mp_engine_qcommon::qfiles::draw_vert_t::drawVert_t;
use mp_engine_qcommon::qfiles::dshader_t::dshader_t;
use mp_engine_qcommon::qfiles::lump_t::lump_t;
use mp_qshared::shared::q_math::PlaneTypeForNormal;
use mp_qshared::shared::q_string::COM_Parse;
use mp_qshared::shared::swap::{LittleFloat, LittleLong};
use mp_qshared::shared::{cplane_t, errorParm_t, MAX_QPATH};

use crate::render_state::frame_state::FrameState;
use crate::render_state::placeholders::WorldAsset;

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
/// transition audit, Group 1: `mnode_t` row). This wave only needs the
/// fields `R_SetParent` touches; `plane`, `firstmarksurface`/
/// `nummarksurfaces`, and the leaf fields land with the `tr_world` waves
/// that build the rest of the node arena.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:917-934`
pub struct Node {
    pub parent: Option<usize>,
    pub children: [Option<usize>; 2],
    /// -1 for nodes, to differentiate from leafs
    pub contents: i32,
}

/// Owned replacement for Raven `srfGridMesh_t`'s vertex data — `verts`
/// becomes an owned `Vec<drawVert_t>` sized by `(width, height)`, replacing
/// the C flexible-array trick (tier-2 transition audit, Group 1:
/// `srfGridMesh_t` row). `widthLodError`/`heightLodError` land with
/// whichever wave ports the LOD-stitching functions that read them.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:750-774`
pub struct GridMesh {
    pub width: i32,
    pub height: i32,
    pub verts: Vec<drawVert_t>,
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
            if (a[0] - b[0]).abs() > 0.1 {
                continue;
            }
            if (a[1] - b[1]).abs() > 0.1 {
                continue;
            }
            if (a[2] - b[2]).abs() > 0.1 {
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
            if (a[0] - b[0]).abs() > 0.1 {
                continue;
            }
            if (a[1] - b[1]).abs() > 0.1 {
                continue;
            }
            if (a[2] - b[2]).abs() > 0.1 {
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
