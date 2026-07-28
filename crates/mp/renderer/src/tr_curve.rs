//! Raven `tr_curve.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_curve.cpp`

// Raven-named functions/types keep their original casing across this
// transcription, matching the rest of the renderer/engine crates.
#![allow(non_snake_case)]

use mp_engine_qcommon::common::Common;
use mp_engine_qcommon::qfiles::draw_vert_t::{drawVert_t, MAXLIGHTMAPS};
use mp_qshared::shared::q_math::{
    _DotProduct, _VectorAdd, _VectorScale, _VectorSubtract, CrossProduct, VectorClear,
    VectorLength, VectorLengthSquared, VectorNormalize, VectorNormalize2,
};
use mp_qshared::shared::vec3_t;
// PORT-NOTE: `native_math` is not yet a direct `mp_renderer` dependency
// (Cargo.toml wiring gap) — `AddPointToBounds`/`ClearBoundsMP` are LAW-cited
// at `crates/native/math/src/qmath.rs` and have no re-export reachable from
// this crate today. Flagged for the integrate phase to add the dependency
// edge; the call sites below are otherwise final.
use native_math::qmath::{AddPointToBounds, ClearBoundsMP};

use crate::render_state::renderer_cvars::RendererCvars;
use crate::tr_local::surface_type_t::surfaceType_t;

/// Raven `MAX_GRID_SIZE` — the bezier-patch control-grid bound.
///
/// Source: `oracle/codemp/renderer/tr_local.h` (`#define MAX_GRID_SIZE 65`,
/// cited by this packet's `R_CreateSurfaceGridMesh`/`InvertErrorTable`
/// oracle signatures, `ctrl[65][65]`/`errorTable[2][65]`).
pub const MAX_GRID_SIZE: usize = 65;

/// A fixed `MAX_GRID_SIZE`x`MAX_GRID_SIZE` bezier control-point buffer —
/// Raven's `drawVert_t ctrl[MAX_GRID_SIZE][MAX_GRID_SIZE]` out-param shape,
/// kept as a fixed array (not `Vec`) since every caller in this file passes
/// the same full-size stack buffer and only `width`/`height` bound the
/// active region.
pub type ControlGrid = [[drawVert_t; MAX_GRID_SIZE]; MAX_GRID_SIZE];

/// Raven's `float errorTable[2][MAX_GRID_SIZE]`.
pub type ErrorTable = [[f32; MAX_GRID_SIZE]; 2];

/// Owned replacement for the tier-2 `srfGridMesh_t`
/// (`crates/mp/renderer/src/tr_local/srf_grid_mesh_s.rs`) — a bezier-patch
/// tessellated surface. The single canonical `srfGridMesh_t` replacement for
/// the renderer: `tr_curve.rs` (tessellation/insertion) and `tr_bsp.rs`
/// (LOD-error fixing, patch stitching) both use this type. Named by DEC-37
/// A13.3: the R2 tier-2 transition audit assigns `srfGridMesh_t`'s pointer
/// fields (`widthLodError`/`heightLodError: *mut f32`, `verts: [drawVert_t;
/// 1]` C flexible-array) to owned `Vec` forms "as each subsystem's logic
/// lands" — `tr_curve.cpp` (this file) is the bezier-patch tessellation
/// subsystem that owns that transition.
///
/// Source: `oracle/codemp/renderer/tr_local.h:750-774`
/// (`renderer-r2-design.md` `### Tier-2 transition audit`, Group 1 —
/// `srfGridMesh_t` row)
// `Clone` added by DEC-43.4: `SurfaceData::Grid` stores a `GridMesh` by value
// in `WorldAsset::surfaces`, which `RenderAssets` clones through
// `Arc::make_mut`.
#[derive(Clone)]
pub struct GridMesh {
    pub surface_type: surfaceType_t,
    pub dlight_bits: i32,
    pub mesh_bounds: [vec3_t; 2],
    pub local_origin: vec3_t,
    pub mesh_radius: f32,
    /// `lodOrigin`.
    pub lod_origin: vec3_t,
    /// `lodRadius`.
    pub lod_radius: f32,
    /// `lodFixed` — `2` once `R_FixSharedVertexLodError_r` (`tr_bsp.rs`) has
    /// stitched this patch's LOD errors against a matching group.
    pub lod_fixed: i32,
    /// `lodStitched` — cleared by `R_StitchPatches` (`tr_bsp.rs`) whenever a
    /// crack fix reshapes the grid, so the patch is revisited.
    pub lod_stitched: i32,
    pub width: i32,
    pub height: i32,
    /// `widthLodError`.
    pub width_lod_error: Vec<f32>,
    /// `heightLodError`.
    pub height_lod_error: Vec<f32>,
    pub verts: Vec<drawVert_t>,
}

/// An empty `GridMesh` placeholder — stands in for a grid that has been moved
/// out of its `Surface` slot to be handed to
/// `R_GridInsertColumn`/`R_GridInsertRow` by value (`tr_bsp::R_StitchPatches`,
/// which repoints `worldData.surfaces[grid2num].data` at the returned grid).
/// Its `SF_BAD` tag makes the transient hole self-evident. A helper rather
/// than a `Default` impl for the same reason as `zero_draw_vert`: `drawVert_t`
/// derives no `Default`.
pub fn empty_grid_mesh() -> GridMesh {
    GridMesh {
        surface_type: surfaceType_t::SF_BAD,
        dlight_bits: 0,
        mesh_bounds: [[0.0; 3]; 2],
        local_origin: [0.0; 3],
        mesh_radius: 0.0,
        lod_origin: [0.0; 3],
        lod_radius: 0.0,
        lod_fixed: 0,
        lod_stitched: 0,
        width: 0,
        height: 0,
        width_lod_error: Vec::new(),
        height_lod_error: Vec::new(),
        verts: Vec::new(),
    }
}

/// The single spelling this file uses for Raven's `temp = ctrl[j][i];`
/// whole-struct `drawVert_t` value copies — a field-wise read, kept as one
/// named helper rather than spread across ~20 call sites.
fn copy_draw_vert(v: &drawVert_t) -> drawVert_t {
    drawVert_t {
        xyz: v.xyz,
        st: v.st,
        lightmap: v.lightmap,
        normal: v.normal,
        color: v.color,
    }
}

/// A zero-valued `drawVert_t` — `drawVert_t` derives no `Default` (it is a
/// tier-1 ABI-adjacent file), so this file spells the zero value once here.
fn zero_draw_vert() -> drawVert_t {
    drawVert_t {
        xyz: [0.0; 3],
        st: [0.0; 2],
        lightmap: [[0.0; 2]; MAXLIGHTMAPS],
        normal: [0.0; 3],
        color: [[0; 4]; MAXLIGHTMAPS],
    }
}

/// Raven `LerpDrawVert`.
///
/// Source: `oracle/codemp/renderer/tr_curve.cpp:27-52`
pub fn LerpDrawVert(a: &drawVert_t, b: &drawVert_t) -> drawVert_t {
    let mut out = zero_draw_vert();

    out.xyz[0] = 0.5 * (a.xyz[0] + b.xyz[0]);
    out.xyz[1] = 0.5 * (a.xyz[1] + b.xyz[1]);
    out.xyz[2] = 0.5 * (a.xyz[2] + b.xyz[2]);

    out.st[0] = 0.5 * (a.st[0] + b.st[0]);
    out.st[1] = 0.5 * (a.st[1] + b.st[1]);

    out.normal[0] = 0.5 * (a.normal[0] + b.normal[0]);
    out.normal[1] = 0.5 * (a.normal[1] + b.normal[1]);
    out.normal[2] = 0.5 * (a.normal[2] + b.normal[2]);

    for k in 0..MAXLIGHTMAPS {
        out.lightmap[k][0] = 0.5 * (a.lightmap[k][0] + b.lightmap[k][0]);
        out.lightmap[k][1] = 0.5 * (a.lightmap[k][1] + b.lightmap[k][1]);

        // Raven's `>>` operates on ints after C integer promotion of the
        // `u8` operands; widen to u16 before the add so the shift matches.
        out.color[k][0] = ((a.color[k][0] as u16 + b.color[k][0] as u16) >> 1) as u8;
        out.color[k][1] = ((a.color[k][1] as u16 + b.color[k][1] as u16) >> 1) as u8;
        out.color[k][2] = ((a.color[k][2] as u16 + b.color[k][2] as u16) >> 1) as u8;
        out.color[k][3] = ((a.color[k][3] as u16 + b.color[k][3] as u16) >> 1) as u8;
    }

    out
}

/// Raven `Transpose`.
///
/// Source: `oracle/codemp/renderer/tr_curve.cpp:59-93`
pub fn Transpose(width: usize, height: usize, ctrl: &mut ControlGrid) {
    if width > height {
        for i in 0..height {
            for j in (i + 1)..width {
                if j < height {
                    // swap the value
                    let temp = copy_draw_vert(&ctrl[j][i]);
                    ctrl[j][i] = copy_draw_vert(&ctrl[i][j]);
                    ctrl[i][j] = temp;
                } else {
                    // just copy
                    ctrl[j][i] = copy_draw_vert(&ctrl[i][j]);
                }
            }
        }
    } else {
        for i in 0..width {
            for j in (i + 1)..height {
                if j < width {
                    // swap the value
                    let temp = copy_draw_vert(&ctrl[i][j]);
                    ctrl[i][j] = copy_draw_vert(&ctrl[j][i]);
                    ctrl[j][i] = temp;
                } else {
                    // just copy
                    ctrl[i][j] = copy_draw_vert(&ctrl[j][i]);
                }
            }
        }
    }
}

/// Raven `neighbors[8][2]` — the bezier-mesh normal-averaging offset table
/// (fn-scope `static` in the oracle; const-table kind per the three-kind
/// rule, so it becomes a plain `const`, no carrier).
///
/// Source: `oracle/codemp/renderer/tr_curve.cpp:116-118`
const NEIGHBORS: [[i32; 2]; 8] = [
    [0, 1],
    [1, 1],
    [1, 0],
    [1, -1],
    [0, -1],
    [-1, -1],
    [-1, 0],
    [-1, 1],
];

/// Raven `MakeMeshNormals`.
///
/// Source: `oracle/codemp/renderer/tr_curve.cpp:103-205`
#[allow(unused_assignments)]
pub fn MakeMeshNormals(width: usize, height: usize, ctrl: &mut ControlGrid) {
    // Raven's `for (i=0;i<height;i++){...break;} if (i==height) wrapWidth=qtrue;`
    // reads the loop counter after the loop to detect "ran to completion
    // without breaking"; tracked directly here instead (porting-rules C10).
    let mut wrap_width = false;
    let mut broke = false;
    for i in 0..height {
        let mut delta: vec3_t = [0.0; 3];
        _VectorSubtract(ctrl[i][0].xyz, ctrl[i][width - 1].xyz, &mut delta);
        let len = VectorLengthSquared(delta);
        if len > 1.0 {
            broke = true;
            break;
        }
    }
    if !broke {
        wrap_width = true;
    }

    let mut wrap_height = false;
    let mut broke = false;
    for i in 0..width {
        let mut delta: vec3_t = [0.0; 3];
        _VectorSubtract(ctrl[0][i].xyz, ctrl[height - 1][i].xyz, &mut delta);
        let len = VectorLengthSquared(delta);
        if len > 1.0 {
            broke = true;
            break;
        }
    }
    if !broke {
        wrap_height = true;
    }

    for i in 0..width {
        for j in 0..height {
            let mut count = 0i32;
            let base: vec3_t = ctrl[j][i].xyz;
            let mut around = [[0.0f32; 3]; 8];
            let mut good = [false; 8];

            for k in 0..8usize {
                VectorClear(&mut around[k]);
                good[k] = false;

                for dist in 1..=3i32 {
                    let mut x = i as i32 + NEIGHBORS[k][0] * dist;
                    let mut y = j as i32 + NEIGHBORS[k][1] * dist;
                    if wrap_width {
                        if x < 0 {
                            x = width as i32 - 1 + x;
                        } else if x >= width as i32 {
                            x = 1 + x - width as i32;
                        }
                    }
                    if wrap_height {
                        if y < 0 {
                            y = height as i32 - 1 + y;
                        } else if y >= height as i32 {
                            y = 1 + y - height as i32;
                        }
                    }

                    if x < 0 || x >= width as i32 || y < 0 || y >= height as i32 {
                        break; // edge of patch
                    }
                    let mut temp: vec3_t = [0.0; 3];
                    _VectorSubtract(ctrl[y as usize][x as usize].xyz, base, &mut temp);
                    if VectorNormalize2(temp, &mut temp) == 0.0 {
                        continue; // degenerate edge, get more dist
                    } else {
                        good[k] = true;
                        around[k] = temp;
                        break; // good edge
                    }
                }
            }

            let mut sum: vec3_t = [0.0; 3];
            VectorClear(&mut sum);
            for k in 0..8usize {
                if !good[k] || !good[(k + 1) & 7] {
                    continue; // didn't get two points
                }
                let mut normal: vec3_t = [0.0; 3];
                CrossProduct(around[(k + 1) & 7], around[k], &mut normal);
                if VectorNormalize2(normal, &mut normal) == 0.0 {
                    continue;
                }
                _VectorAdd(normal, sum, &mut sum);
                count += 1;
            }
            if count == 0 {
                // Raven: //printf("bad normal\n");
                count = 1;
            }
            VectorNormalize2(sum, &mut ctrl[j][i].normal);
        }
    }
}

/// Raven `InvertCtrl`.
///
/// Source: `oracle/codemp/renderer/tr_curve.cpp:212-223`
pub fn InvertCtrl(width: usize, height: usize, ctrl: &mut ControlGrid) {
    for i in 0..height {
        for j in 0..(width / 2) {
            let temp = copy_draw_vert(&ctrl[i][j]);
            ctrl[i][j] = copy_draw_vert(&ctrl[i][width - 1 - j]);
            ctrl[i][width - 1 - j] = temp;
        }
    }
}

/// Raven `InvertErrorTable`.
///
/// Source: `oracle/codemp/renderer/tr_curve.cpp:230-244`
pub fn InvertErrorTable(error_table: &mut ErrorTable, width: usize, height: usize) {
    // Raven: `Com_Memcpy(copy, errorTable, sizeof(copy));` — a fixed-size
    // `f32` array is `Copy`, so the stack-buffer duplication is a plain
    // value copy (porting-rules C9: manual copy collapses into ownership).
    let copy = *error_table;

    for i in 0..width {
        error_table[1][i] = copy[0][i]; //[width-1-i];
    }

    for i in 0..height {
        error_table[0][i] = copy[1][height - 1 - i];
    }
}

/// Raven `R_CreateSurfaceGridMesh`.
///
/// Source: `oracle/codemp/renderer/tr_curve.cpp:279-331`
// PORT-NOTE: the oracle's `#ifdef PATCH_STITCHING`/`#else` branches both
// size one heap block (`Z_Malloc`/`Hunk_Alloc`), `Com_Memset` it, then fill
// `widthLodError`/`heightLodError` with a second/third allocation each.
// `GridMesh`'s owned `Vec` fields replace all of that manual sizing/zeroing
// (porting-rules C9) — no `Z_Malloc`/`Hunk_Alloc`/`Com_Memset`/`memtag_t`
// call is needed; the struct is built directly from Rust values.
pub fn R_CreateSurfaceGridMesh(
    width: usize,
    height: usize,
    ctrl: &ControlGrid,
    error_table: &ErrorTable,
) -> GridMesh {
    let width_lod_error = error_table[0][..width].to_vec();
    let height_lod_error = error_table[1][..height].to_vec();

    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];
    // PORT-NOTE: the packet flags `ClearBounds` as unresolved (MP/SP fork
    // ambiguity) — `crates/mp/renderer` is the MP crate, so the MP fork
    // (`ClearBoundsMP`, `crates/native/math/src/qmath.rs`) is the matching
    // pick, mirroring the established `ClearBoundsMP as ClearBounds`
    // pattern already used by `mp_game`'s `q_math` module.
    ClearBoundsMP(&mut mins, &mut maxs);

    let mut verts: Vec<drawVert_t> = (0..width * height).map(|_| zero_draw_vert()).collect();
    for i in 0..width {
        for j in 0..height {
            let vert = copy_draw_vert(&ctrl[j][i]);
            AddPointToBounds(vert.xyz, &mut mins, &mut maxs);
            verts[j * width + i] = vert;
        }
    }

    // compute local origin and bounds
    let mut local_origin: vec3_t = [0.0; 3];
    _VectorAdd(mins, maxs, &mut local_origin);
    _VectorScale(local_origin, 0.5, &mut local_origin);
    let mut tmp_vec: vec3_t = [0.0; 3];
    _VectorSubtract(mins, local_origin, &mut tmp_vec);
    let mesh_radius = VectorLength(tmp_vec);

    GridMesh {
        surface_type: surfaceType_t::SF_GRID,
        dlight_bits: 0,
        mesh_bounds: [mins, maxs],
        local_origin,
        mesh_radius,
        lod_origin: local_origin,
        lod_radius: mesh_radius,
        lod_fixed: 0,
        lod_stitched: 0,
        width: width as i32,
        height: height as i32,
        width_lod_error,
        height_lod_error,
        verts,
    }
}

/// Raven `R_FreeSurfaceGridMesh`.
///
/// Source: `oracle/codemp/renderer/tr_curve.cpp:338-342`
// PORT-NOTE: Raven explicitly `Z_Free`s `widthLodError`, `heightLodError`,
// and `grid` itself — three heap blocks under the old Hunk/Zone model.
// `GridMesh`'s `Vec`/owned fields hold that storage (porting-rules C9);
// dropping the value frees it, so consuming `grid` by value replaces the
// three explicit frees.
pub fn R_FreeSurfaceGridMesh(grid: GridMesh) {
    drop(grid);
}

/// Raven `PutPointsOnCurve`.
///
/// Source: `oracle/codemp/renderer/tr_curve.cpp:251-272`
pub fn PutPointsOnCurve(ctrl: &mut ControlGrid, width: usize, height: usize) {
    for i in 0..width {
        let mut j = 1usize;
        while j < height {
            let prev = LerpDrawVert(&ctrl[j][i], &ctrl[j + 1][i]);
            let next = LerpDrawVert(&ctrl[j][i], &ctrl[j - 1][i]);
            ctrl[j][i] = LerpDrawVert(&prev, &next);
            j += 2;
        }
    }

    for j in 0..height {
        let mut i = 1usize;
        while i < width {
            let prev = LerpDrawVert(&ctrl[j][i], &ctrl[j][i + 1]);
            let next = LerpDrawVert(&ctrl[j][i], &ctrl[j][i - 1]);
            ctrl[j][i] = LerpDrawVert(&prev, &next);
            i += 2;
        }
    }
}

/// A fresh, zero-filled `ControlGrid` — `drawVert_t` has no `Copy`/`Clone`
/// derive (see `copy_draw_vert`), so the fixed `[[T; 65]; 65]` cannot be
/// built with array-repeat syntax; `array::from_fn` fills it element-wise
/// instead. Stands in for the oracle's uninitialized `MAC_STATIC drawVert_t
/// ctrl[MAX_GRID_SIZE][MAX_GRID_SIZE]` stack buffer.
fn zero_control_grid() -> ControlGrid {
    std::array::from_fn(|_| std::array::from_fn(|_| zero_draw_vert()))
}

/// Raven `R_GridInsertColumn`.
///
/// Source: `oracle/codemp/renderer/tr_curve.cpp:511-558`
pub fn R_GridInsertColumn(
    grid: GridMesh,
    column: usize,
    row: usize,
    point: vec3_t,
    loderror: f32,
) -> Option<GridMesh> {
    let old_width = grid.width as usize;
    let height = grid.height as usize;
    let width = old_width + 1;
    if width > MAX_GRID_SIZE {
        return None;
    }

    let mut ctrl = zero_control_grid();
    let mut error_table: ErrorTable = [[0.0; MAX_GRID_SIZE]; 2];

    let mut old_column = 0usize;
    for i in 0..width {
        if i == column {
            // insert new column
            for j in 0..height {
                // PORT-NOTE (porting-rules §19): Raven's straddle read
                // `grid->verts[j*grid->width + i-1]`/`[...+i]` can index
                // outside the old array when inserting at a boundary
                // (column == 0 or == old width) — C UB (stack over-read).
                // Rust's `Vec` bounds check turns that into a panic instead,
                // the defined-behavior pick for this UB site.
                let mut v = LerpDrawVert(
                    &grid.verts[j * old_width + i - 1],
                    &grid.verts[j * old_width + i],
                );
                if j == row {
                    v.xyz = point;
                }
                ctrl[j][i] = v;
            }
            error_table[0][i] = loderror;
            continue;
        }
        error_table[0][i] = grid.width_lod_error[old_column];
        for j in 0..height {
            ctrl[j][i] = copy_draw_vert(&grid.verts[j * old_width + old_column]);
        }
        old_column += 1;
    }
    for j in 0..height {
        error_table[1][j] = grid.height_lod_error[j];
    }
    // put all the aproximating points on the curve
    //PutPointsOnCurve( ctrl, width, height );
    // calculate normals
    MakeMeshNormals(width, height, &mut ctrl);

    let lod_origin = grid.lod_origin;
    let lod_radius = grid.lod_radius;
    // free the old grid
    R_FreeSurfaceGridMesh(grid);
    // create a new grid
    let mut new_grid = R_CreateSurfaceGridMesh(width, height, &ctrl, &error_table);
    new_grid.lod_radius = lod_radius;
    new_grid.lod_origin = lod_origin;
    Some(new_grid)
}

/// Raven `R_GridInsertRow`.
///
/// Source: `oracle/codemp/renderer/tr_curve.cpp:565-612`
pub fn R_GridInsertRow(
    grid: GridMesh,
    row: usize,
    column: usize,
    point: vec3_t,
    loderror: f32,
) -> Option<GridMesh> {
    let width = grid.width as usize;
    let old_height = grid.height as usize;
    let height = old_height + 1;
    if height > MAX_GRID_SIZE {
        return None;
    }

    let mut ctrl = zero_control_grid();
    let mut error_table: ErrorTable = [[0.0; MAX_GRID_SIZE]; 2];

    let mut old_row = 0usize;
    for i in 0..height {
        if i == row {
            // insert new row
            for j in 0..width {
                // PORT-NOTE (porting-rules §19): Raven's straddle read
                // `grid->verts[(i-1)*grid->width + j]`/`[i*grid->width+j]`
                // can index outside the old array when inserting at a
                // boundary (row == 0 or == old height) — C UB (stack
                // over-read). Rust's `Vec` bounds check turns that into a
                // panic instead, the defined-behavior pick for this UB site.
                let mut v =
                    LerpDrawVert(&grid.verts[(i - 1) * width + j], &grid.verts[i * width + j]);
                if j == column {
                    v.xyz = point;
                }
                ctrl[i][j] = v;
            }
            error_table[1][i] = loderror;
            continue;
        }
        error_table[1][i] = grid.height_lod_error[old_row];
        for j in 0..width {
            ctrl[i][j] = copy_draw_vert(&grid.verts[old_row * width + j]);
        }
        old_row += 1;
    }
    for j in 0..width {
        error_table[0][j] = grid.width_lod_error[j];
    }
    // put all the aproximating points on the curve
    //PutPointsOnCurve( ctrl, width, height );
    // calculate normals
    MakeMeshNormals(width, height, &mut ctrl);

    let lod_origin = grid.lod_origin;
    let lod_radius = grid.lod_radius;
    // free the old grid
    R_FreeSurfaceGridMesh(grid);
    // create a new grid
    let mut new_grid = R_CreateSurfaceGridMesh(width, height, &ctrl, &error_table);
    new_grid.lod_radius = lod_radius;
    new_grid.lod_origin = lod_origin;
    Some(new_grid)
}

/// Raven `R_SubdividePatchToGrid`.
///
/// `points` is Raven's `drawVert_t points[MAX_PATCH_SIZE*MAX_PATCH_SIZE]`
/// out-of-packet fixed array — kept as a slice (translation dictionary:
/// array param → slice); every index used below (`points[j*width+i]`) stays
/// in range for any `width`/`height` the caller passes, matching the
/// oracle's own unchecked indexing.
///
/// `r_subdivisions` reads through the live engine cvar table
/// (`RendererCvars::r_subdivisions`, DEC-37 A13.1 — this packet's STATE
/// HOMES row), the `tr_main.rs` `R_SetupProjection` `common.cvar(handle)`
/// precedent.
///
/// Source: `oracle/codemp/renderer/tr_curve.cpp:349-504`
pub fn R_SubdividePatchToGrid(
    mut width: usize,
    mut height: usize,
    points: &[drawVert_t],
    common: &Common,
    cvars: &RendererCvars,
) -> GridMesh {
    let mut ctrl = zero_control_grid();
    let mut error_table: ErrorTable = [[0.0; MAX_GRID_SIZE]; 2];

    for i in 0..width {
        for j in 0..height {
            ctrl[j][i] = copy_draw_vert(&points[j * width + i]);
        }
    }

    let r_subdivisions_value = common.cvar(cvars.r_subdivisions).value;

    for dir in 0..2usize {
        for j in 0..MAX_GRID_SIZE {
            error_table[dir][j] = 0.0;
        }

        // horizontal subdivisions
        let mut j: usize = 0;
        while j + 2 < width {
            // check subdivided midpoints against control points

            // FIXME: also check midpoints of adjacent patches against the control points
            // this would basically stitch all patches in the same LOD group together.

            let mut max_len: f32 = 0.0;
            for i in 0..height {
                // calculate the point on the curve
                let mut midxyz: vec3_t = [0.0; 3];
                for l in 0..3usize {
                    midxyz[l] =
                        (ctrl[i][j].xyz[l] + ctrl[i][j + 1].xyz[l] * 2.0 + ctrl[i][j + 2].xyz[l])
                            * 0.25;
                }

                // see how far off the line it is
                // using dist-from-line will not account for internal
                // texture warping, but it gives a lot less polygons than
                // dist-from-midpoint
                _VectorSubtract(midxyz, ctrl[i][j].xyz, &mut midxyz);
                let mut dir_vec: vec3_t = [0.0; 3];
                _VectorSubtract(ctrl[i][j + 2].xyz, ctrl[i][j].xyz, &mut dir_vec);
                VectorNormalize(&mut dir_vec);

                let d = _DotProduct(midxyz, dir_vec);
                let mut projected: vec3_t = [0.0; 3];
                _VectorScale(dir_vec, d, &mut projected);
                _VectorSubtract(midxyz, projected, &mut midxyz);
                let len = VectorLengthSquared(midxyz); // we will do the sqrt later

                if len > max_len {
                    max_len = len;
                }
            }

            // C `sqrt()` promotes its argument to double; f64 intermediate
            // per wave-0 ruling 12, rounded to f32 once at the assignment
            // (C's own narrowing point).
            max_len = (max_len as f64).sqrt() as f32;
            // if all the points are on the lines, remove the entire columns
            if max_len < 0.1 {
                error_table[dir][j + 1] = 999.0;
                j += 2;
                continue;
            }

            // see if we want to insert subdivided columns
            if width + 2 > MAX_GRID_SIZE {
                error_table[dir][j + 1] = 1.0 / max_len;
                j += 2;
                continue; // can't subdivide any more
            }

            if max_len <= r_subdivisions_value {
                error_table[dir][j + 1] = 1.0 / max_len;
                j += 2;
                continue; // didn't need subdivision
            }

            error_table[dir][j + 2] = 1.0 / max_len;

            // insert two columns and replace the peak
            width += 2;
            for i in 0..height {
                let prev = LerpDrawVert(&ctrl[i][j], &ctrl[i][j + 1]);
                let next = LerpDrawVert(&ctrl[i][j + 1], &ctrl[i][j + 2]);
                let mid = LerpDrawVert(&prev, &next);

                let mut k = width - 1;
                while k > j + 3 {
                    ctrl[i][k] = copy_draw_vert(&ctrl[i][k - 2]);
                    k -= 1;
                }
                ctrl[i][j + 1] = prev;
                ctrl[i][j + 2] = mid;
                ctrl[i][j + 3] = next;
            }

            // back up and recheck this set again, it may need more
            // subdivision. Raven's `j -= 2;` here is immediately followed by
            // the C `for` loop's own `j += 2` increment, netting `j`
            // unchanged; this `while` form reproduces that by simply not
            // advancing `j` on this path.
        }

        Transpose(width, height, &mut ctrl);
        let t = width;
        width = height;
        height = t;
    }

    // put all the aproximating points on the curve
    PutPointsOnCurve(&mut ctrl, width, height);

    // cull out any rows or columns that are colinear
    // `i + 1 < width` rather than `i < width - 1`: the C `int` comparison is
    // safe at `width == 0`, the `usize` subtraction would underflow. Same
    // predicate for every reachable `width`.
    // Source: `oracle/codemp/renderer/tr_curve.cpp:460`
    let mut i: usize = 1;
    while i + 1 < width {
        if error_table[0][i] == 999.0 {
            let mut j = i + 1;
            while j < width {
                for k in 0..height {
                    ctrl[k][j - 1] = copy_draw_vert(&ctrl[k][j]);
                }
                error_table[0][j - 1] = error_table[0][j];
                j += 1;
            }
            width -= 1;
        }
        i += 1;
    }

    // `i + 1 < height`: underflow guard, as above.
    // Source: `oracle/codemp/renderer/tr_curve.cpp:473`
    let mut i: usize = 1;
    while i + 1 < height {
        if error_table[1][i] == 999.0 {
            let mut j = i + 1;
            while j < height {
                for k in 0..width {
                    ctrl[j - 1][k] = copy_draw_vert(&ctrl[j][k]);
                }
                error_table[1][j - 1] = error_table[1][j];
                j += 1;
            }
            height -= 1;
        }
        i += 1;
    }

    // flip for longest tristrips as an optimization
    // the results should be visually identical with or
    // without this step
    if height > width {
        Transpose(width, height, &mut ctrl);
        InvertErrorTable(&mut error_table, width, height);
        let t = width;
        width = height;
        height = t;
        InvertCtrl(width, height, &mut ctrl);
    }

    // calculate normals
    MakeMeshNormals(width, height, &mut ctrl);

    R_CreateSurfaceGridMesh(width, height, &ctrl, &error_table)
}
