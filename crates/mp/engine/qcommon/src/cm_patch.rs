//! `CCMPatch` — one terrain collision patch (`CmPatch`, ruling 40 rename), its
//! **own file** per ruling 39d / §21 (one class per file), beside
//! `cm_terrain.rs`. Built by the LIVE `CmLandScape::UpdatePatches`
//! (`cm_terrain.cpp:898-927`) and owned as `CmLandScape.patches: Vec<CmPatch>`
//! (frozen there, not here — Files roster, class `CCMLandScape`). Not in any
//! pub Seam signature (`docs/subsystems/rmg-terrain.md` Files roster,
//! `cm_patch.rs` row): every method below is a private helper the owning
//! `CmLandScape` (and, cross-file, the `cm`-C-track `CM_HandlePatchCollision`,
//! Open questions below) calls intra-crate.
//!
//! Per `docs/subsystems/rmg-terrain.md` (roster row, class `CCMPatch`;
//! RMG-D7/ruling 46) only **five** prototypes are LIVE and get stubs below:
//! `Init`, `InitPlane`, `CreatePatchPlaneData`, `GetAdjacentBrushX`,
//! `GetAdjacentBrushY` (`cm_landscape.h:125-130`).
//!
//! **§B3/RMG-D4h — `owner: CCMLandScape*` (`cm_landscape.h:93`) is DROPPED.**
//! It is a live back-pointer into state owned elsewhere (read by
//! `GetAdjacentBrushY`, `cm_terrain.cpp:246-256`); per §B3 no `CmPatch` field
//! aliases it — the owning `CmLandScape` is threaded as a `&`/`&mut` parameter
//! into every method that used `owner->…` (Init, CreatePatchPlaneData,
//! GetAdjacentBrushX/Y).
//!
//! **RMG-D7/ruling 46 — `mPatchBrushData` (`cbrush_s*`, `cm_landscape.h:100`)
//! is an offset/length RANGE into `CmLandScape`'s single shared `Vec`-backed
//! brush arena**, not a raw pointer (§B5) — see `brush_offset`/`brush_len`
//! below.
//!
//! **§20-dropped (zero-caller or generation-path-only), module-doc note, no
//! stub:**
//! - `CCMPatch(void) {}` (`cm_landscape.h:105`) and `~CCMPatch(void)`
//!   (def `cm_terrain.cpp:112-114`) — both empty bodies, no member is
//!   initialized or released. `#[derive(Default)]` below is the faithful
//!   Rust equivalent of the no-op ctor (§19: Raven leaves these members
//!   uninitialized garbage until `Init` runs; `Default` zero-inits instead —
//!   never reads as garbage, a definedness improvement, not a behavior
//!   change on the live path); the no-op dtor needs no `Drop` impl.
//! - `GetWorld` (`:109`) — zero callers anywhere in codemp.
//! - `GetMins`/`GetMaxs` (`:110-111`, the patch's own, distinct from
//!   `CCMLandScape`'s same-named accessors at `:200-201`) and
//!   `GetHeightMapX`/`GetHeightMapY`/`GetHeight(corner)` (`:113-115`) — their
//!   only callers are `RM_Terrain.cpp:343-344,385,467,470`, the ruling-17
//!   §20-dropped client-model chain (RMG-D4c), dead under DEDICATED (RMG-D1).
//! - `SetSurfaceFlags`/`GetSurfaceFlags`/`SetContents`/`GetContents`
//!   (`:119-122`) — zero callers anywhere (a grep of `->SetSurfaceFlags`,
//!   `->SetContents`, and the patch's own no-arg `GetSurfaceFlags()`/
//!   `GetContents()` finds none; the same-named calls that do exist resolve
//!   to `CCMHeightDetails`'s or `CCMLandScape`'s own overloads,
//!   `cm_landscape.h:225-226`, `cm_terrain.cpp:32`).
//!
//! Source: `oracle/codemp/qcommon/cm_landscape.h:90-131`,
//! `oracle/codemp/qcommon/cm_terrain.cpp`

use crate::cm::cbrush_s::cbrush_t;
use crate::cm::cbrushside_s::cbrushside_t;
use crate::cm_terrain::CmLandScape;
use mp_qshared::shared::collision::{cplane_t, PLANE_X, PLANE_Y, PLANE_Z};
use mp_qshared::shared::{vec3_t, vec3pair_t, VectorNormalize};

/// Raven `BRUSH_SIDES_PER_TERXEL` under the unconditionally-defined
/// `_SMOOTH_TERXEL_BRUSH` (no `#undef`/config gate anywhere in the TU, so the
/// `#ifndef _SMOOTH_TERXEL_BRUSH` arm of every method below is preprocessor-
/// excluded dead code and is not transcribed).
///
/// Source: `oracle/codemp/qcommon/cm_terrain.cpp:17-22`
const BRUSH_SIDES_PER_TERXEL: usize = 8;

/// Raven `MAX_WORLD_COORD`.
/// Source: `oracle/codemp/game/q_shared.h:18`
const MAX_WORLD_COORD: f32 = 64.0 * 1024.0;

/// Raven `MIN_WORLD_COORD`.
/// Source: `oracle/codemp/game/q_shared.h:19`
const MIN_WORLD_COORD: f32 = -64.0 * 1024.0;

/// Raven `PLANE_NON_AXIAL` (`PLANE_X`/`PLANE_Y`/`PLANE_Z` are already ported,
/// `mp_qshared::shared::collision`; this fourth band has no existing home
/// there, so it is repeated locally rather than widening that module's
/// public surface for one private-helper use).
/// Source: `oracle/codemp/game/q_shared.h:1847`
const PLANE_NON_AXIAL: i32 = 3;

/// Raven `DotProduct` macro. Pure vec3 math with no existing home reachable
/// from this crate (`mp_engine_qcommon` depends on `mp_qshared`/
/// `mp_host_interface` only — `q_math.c`'s free-fn port lives in the
/// game-tier `mp_game` crate, wrong dependency direction, porting-rules
/// workspace-architecture tiers), so the handful of primitives
/// `InitPlane`/`CreatePatchPlaneData` need are transcribed locally.
/// Source: `oracle/codemp/game/q_shared.h:1358`
fn dot_product(a: vec3_t, b: vec3_t) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Raven `VectorSubtract` macro.
/// Source: `oracle/codemp/game/q_shared.h:1359`
fn vector_subtract(a: vec3_t, b: vec3_t) -> vec3_t {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// Raven `CrossProduct`.
/// Source: `oracle/codemp/game/q_shared.h:1553-1557`
fn cross_product(v1: vec3_t, v2: vec3_t) -> vec3_t {
    [
        v1[1] * v2[2] - v1[2] * v2[1],
        v1[2] * v2[0] - v1[0] * v2[2],
        v1[0] * v2[1] - v1[1] * v2[0],
    ]
}

/// Raven `PlaneTypeForNormal` macro.
/// Source: `oracle/codemp/game/q_shared.h:1856`
fn plane_type_for_normal(n: vec3_t) -> u8 {
    if n[0] == 1.0 {
        PLANE_X as u8
    } else if n[1] == 1.0 {
        PLANE_Y as u8
    } else if n[2] == 1.0 {
        PLANE_Z as u8
    } else {
        PLANE_NON_AXIAL as u8
    }
}

/// Raven `SetPlaneSignbits`.
/// Source: `oracle/codemp/game/q_math.c:751-762`
fn set_plane_signbits(plane: &mut cplane_t) {
    let mut bits: u8 = 0;
    for j in 0..3 {
        if plane.normal[j] < 0.0 {
            bits |= 1 << j;
        }
    }
    plane.signbits = bits;
}

/// Byte length of one patch's slice of the shared brush arena (RMG-D7):
/// `numBrushesPerPatch * sizeof(cbrush_t) + numBrushesPerPatch *
/// BRUSH_SIDES_PER_TERXEL * 2 * (sizeof(cbrushside_t) + sizeof(cplane_t))` —
/// the same formula the ctor/`UpdatePatches` use to size/slice the shared
/// `Z_Malloc(size * GetBlockCount())` buffer, recomputed here from `terxels`
/// (a landscape-wide constant, identical for every patch) since `Init`'s
/// frozen signature receives only the starting `brush_offset`, not a length.
///
/// Source: `oracle/codemp/qcommon/cm_terrain.cpp:212-215`
fn brush_region_len(terxels: i32) -> usize {
    let num_brushes_per_patch = (terxels * terxels * 2) as usize;
    num_brushes_per_patch * core::mem::size_of::<cbrush_t>()
        + num_brushes_per_patch
            * BRUSH_SIDES_PER_TERXEL
            * 2
            * (core::mem::size_of::<cbrushside_t>() + core::mem::size_of::<cplane_t>())
}

/// Pure index math for `GetAdjacentBrushY`: `Some((blockX, blockY))` when the
/// y-adjacent terxel crosses into a different patch (Raven's
/// `owner->GetPatch(...)` branch), `None` when it stays within `self`
/// (Raven's `patch = this;`); plus the target brush's index within its
/// patch's 2-brushes-per-terxel array.
///
/// Source: `oracle/codemp/qcommon/cm_terrain.cpp:246-256`
fn adjacent_brush_y_index(terxels: i32, x: i32, y: i32) -> (Option<(i32, i32)>, usize) {
    let yo1 = y % terxels;
    let yo2 = (y - 1) % terxels;
    let xo = x % terxels;
    let other_patch = (yo2 > yo1).then(|| (x / terxels, (y - 1) / terxels));
    let index = ((yo2 * terxels + xo) * 2 + 1) as usize;
    (other_patch, index)
}

/// Pure index math for `GetAdjacentBrushX` — see
/// [`adjacent_brush_y_index`] for the shape rationale.
///
/// Source: `oracle/codemp/qcommon/cm_terrain.cpp:272-282`
fn adjacent_brush_x_index(terxels: i32, x: i32, y: i32) -> (Option<(i32, i32)>, usize) {
    let xo1 = x % terxels;
    let xo2 = (x - 1) % terxels;
    let yo = y % terxels;
    let other_patch = (xo2 > xo1).then(|| ((x - 1) / terxels, y / terxels));
    let mut index = ((yo * terxels + xo2) * 2) as usize;
    if (x + y) & 1 == 0 {
        index += 1;
    }
    (other_patch, index)
}

/// Which of the 4 corner heightmap-sample offsets around terxel `(x, y)` map
/// to TL/TR/BL/BR, permuted by the checkerboard `(x+y)&1` split (Raven splits
/// each terxel into 2 triangles along whichever diagonal keeps the split
/// consistent across the grid).
///
/// Source: `oracle/codemp/qcommon/cm_terrain.cpp:325-341`
fn terxel_corner_offsets(x: i32, y: i32, real_width: i32) -> [i32; 4] {
    let tl = y * real_width + x;
    let tr = y * real_width + x + 1;
    let bl = (y + 1) * real_width + x;
    let br = (y + 1) * real_width + x + 1;
    if (x + y) & 1 != 0 {
        [tl, tr, bl, br]
    } else {
        [tr, br, tl, bl]
    }
}

/// Raw-pointer accessor into the shared brush arena (RMG-D7/§B5): the
/// `index`th `cbrush_t` within the byte range starting at `base_offset`.
/// Mirrors Raven's `mPatchBrushData + index` pointer arithmetic over the
/// `Z_Malloc`'d buffer.
///
/// # Safety
/// `arena_ptr` must be a valid pointer into a byte buffer of at least
/// `base_offset + (index + 1) * size_of::<cbrush_t>()` bytes, and no other
/// live reference may alias the addressed `cbrush_t`.
unsafe fn brush_ptr(arena_ptr: *mut u8, base_offset: usize, index: usize) -> *mut cbrush_t {
    arena_ptr.add(base_offset + index * core::mem::size_of::<cbrush_t>()) as *mut cbrush_t
}

/// Raw-pointer accessor into the shared brush arena — the `index`th
/// `cbrushside_t` within the byte range starting at `base_offset`. See
/// [`brush_ptr`] for the safety contract.
unsafe fn side_ptr(arena_ptr: *mut u8, base_offset: usize, index: usize) -> *mut cbrushside_t {
    arena_ptr.add(base_offset + index * core::mem::size_of::<cbrushside_t>()) as *mut cbrushside_t
}

/// Raw-pointer accessor into the shared brush arena — the `index`th
/// `cplane_t` within the byte range starting at `base_offset`. See
/// [`brush_ptr`] for the safety contract.
unsafe fn plane_ptr(arena_ptr: *mut u8, base_offset: usize, index: usize) -> *mut cplane_t {
    arena_ptr.add(base_offset + index * core::mem::size_of::<cplane_t>()) as *mut cplane_t
}

/// Copies one `cbrushside_t`'s fields (Raven's `memcpy(dst, src,
/// sizeof(cbrushside_t))`).
///
/// # Safety
/// `dst` and `src` must each be valid, non-aliasing pointers to a live
/// `cbrushside_t`.
unsafe fn copy_side(dst: *mut cbrushside_t, src: *mut cbrushside_t) {
    (*dst).plane = (*src).plane;
    (*dst).shaderNum = (*src).shaderNum;
}

/// `CCMPatch` — one collision patch of the terrain brush grid.
///
/// **Complete field set.** `owner` (`cm_landscape.h:93`) is dropped (§B3,
/// above) — its owning `CmLandScape` is threaded per-call instead.
/// `mPatchBrushData` (`:100`) becomes `brush_offset`/`brush_len` (RMG-D7): an
/// offset/length range into `CmLandScape`'s shared byte arena, mirroring
/// Raven's per-patch pointer-arithmetic slice
/// (`cm_terrain.cpp:213-215,319-321,524,588,925`) — no raw pointer, §B5.
/// `mHeightMap` (`:95`, `byte*` into the landscape's own height map,
/// `cm_terrain.cpp:538`) is the same "pointer into state owned elsewhere"
/// shape as `mPatchBrushData`, so it becomes `height_map_offset` for the same
/// reason, though it is write-only within the live `Init` body (read nowhere
/// else, `cm_terrain.cpp:578-581` only).
///
/// Source: `oracle/codemp/qcommon/cm_landscape.h:90-102`
#[derive(Default)]
pub struct CmPatch {
    /// `mHx` — terxel x coord of this patch's top-left corner.
    /// Source: `oracle/codemp/qcommon/cm_landscape.h:94`
    pub hx: i32,
    /// `mHy` — terxel y coord of this patch's top-left corner.
    /// Source: `oracle/codemp/qcommon/cm_landscape.h:94`
    pub hy: i32,
    /// `mHeightMap` — offset into the owning `CmLandScape`'s height map
    /// (§B3-style pointer-into-owner reshape; write-only past `Init`).
    /// Source: `oracle/codemp/qcommon/cm_landscape.h:95`
    pub height_map_offset: usize,
    /// `mCornerHeights[4]` — heights at the corners of the patch.
    /// Source: `oracle/codemp/qcommon/cm_landscape.h:96`
    pub corner_heights: [u8; 4],
    /// `mWorldCoords` — world coordinate offset of this patch.
    /// Source: `oracle/codemp/qcommon/cm_landscape.h:97`
    pub world_coords: vec3_t,
    /// `mBounds` — mins/maxs of the patch for culling.
    /// Source: `oracle/codemp/qcommon/cm_landscape.h:98`
    pub bounds: vec3pair_t,
    /// `mNumBrushes` — number of brushes to collide with in the patch.
    /// Source: `oracle/codemp/qcommon/cm_landscape.h:99`
    pub num_brushes: i32,
    /// `mPatchBrushData` (RMG-D7/ruling 46) — byte offset of this patch's
    /// slice within the owning `CmLandScape`'s shared brush arena.
    /// Source: `oracle/codemp/qcommon/cm_landscape.h:100`
    pub brush_offset: usize,
    /// `mPatchBrushData` (RMG-D7/ruling 46) — byte length of this patch's
    /// slice within the owning `CmLandScape`'s shared brush arena.
    /// Source: `oracle/codemp/qcommon/cm_landscape.h:100`
    pub brush_len: usize,
    /// `mSurfaceFlags` — surfaceflag of the heightshader.
    /// Source: `oracle/codemp/qcommon/cm_landscape.h:101`
    pub surface_flags: i32,
    /// `mContentFlags` — contents of the heightshader.
    /// Source: `oracle/codemp/qcommon/cm_landscape.h:102`
    pub content_flags: i32,
}

impl CmPatch {
    /// `CCMPatch::Init` — sets the patch's world/height/bounds/corner/shader
    /// data from the owning landscape and this patch's slice of the shared
    /// height map and brush arena, then builds the patch's collision planes
    /// (`CreatePatchPlaneData`, `:589`). `ls` is `&mut` because `Init` calls
    /// through to `CreatePatchPlaneData`, which writes into `ls`'s shared
    /// brush arena (RMG-D7). `height_map` is the landscape's whole height map
    /// (Raven's `hMap` param, passed un-offset, `cm_terrain.cpp:925`);
    /// `brush_offset` is this patch's pre-computed slice start within the
    /// shared arena (Raven's `patchBrushData` param, already offset by the
    /// caller, `cm_terrain.cpp:925`).
    ///
    /// Source: `oracle/codemp/qcommon/cm_terrain.cpp:524-591`
    pub fn init(
        &mut self,
        ls: &mut CmLandScape,
        height_x: i32,
        height_y: i32,
        world: vec3_t,
        height_map: &[u8],
        brush_offset: usize,
    ) {
        // Store the base of the top left corner.
        self.world_coords = world;

        // Store pointer to first byte of the height data for this patch.
        self.hx = height_x;
        self.hy = height_y;
        let real_width = ls.width + 1; // owner->GetRealWidth()
        self.height_map_offset = (height_y * real_width + height_x) as usize;

        // Calculate the bounds for culling. Use the dimensions 1 terxel
        // outside the patch to allow for sloping of edge terxels.
        let mut min: i32 = 256;
        let mut max: i32 = -1;
        for y in (height_y - 1)..(height_y + ls.terxels + 1) {
            if y < 0 {
                continue;
            }
            for x in (height_x - 1)..(height_x + ls.terxels + 1) {
                if x < 0 {
                    continue;
                }
                let height = height_map[(y * real_width + x) as usize] as i32;
                if height > max {
                    max = height;
                }
                if height < min {
                    min = height;
                }
            }
        }

        // Mins.
        self.bounds[0][0] = world[0];
        self.bounds[0][1] = world[1];
        self.bounds[0][2] = world[2] + (min as f32) * ls.terxel_size[2];

        // Maxs.
        self.bounds[1][0] = world[0] + ls.patch_size[0];
        self.bounds[1][1] = world[1] + ls.patch_size[1];
        self.bounds[1][2] = world[2] + (max as f32) * ls.terxel_size[2];

        // Corner heights.
        let terxels = ls.terxels as usize;
        let hm = self.height_map_offset;
        self.corner_heights[0] = height_map[hm];
        self.corner_heights[1] = height_map[hm + terxels];
        self.corner_heights[2] = height_map[hm + terxels * real_width as usize];
        self.corner_heights[3] = height_map[hm + terxels * real_width as usize + terxels];

        // Set the surfaceFlags using average height (may want a more
        // complex algo here).
        let avg = ((min + max) >> 1) as usize;
        self.surface_flags = ls.height_details[avg].get_surface_flags();
        self.content_flags = ls.height_details[avg].get_contents();

        // Set base of brush data from big array.
        self.brush_offset = brush_offset;
        self.brush_len = brush_region_len(ls.terxels);
        self.create_patch_plane_data(ls);
    }

    /// `CCMPatch::InitPlane` — initializes a `cbrushside_t`/`cplane_t` pair
    /// from 3 world-space corner coords. Does not touch `self` (Raven's
    /// method is a non-static member of `CCMPatch` that never reads `this`);
    /// kept as an instance method for fidelity to the class it is declared on
    /// (`docs/subsystems/rmg-terrain.md` roster: "`CmPatch::Init`/`InitPlane`/
    /// `CreatePatchPlaneData`").
    ///
    /// Source: `oracle/codemp/qcommon/cm_terrain.cpp:223-241`
    pub fn init_plane(
        &self,
        side: &mut cbrushside_t,
        plane: &mut cplane_t,
        p0: vec3_t,
        p1: vec3_t,
        p2: vec3_t,
    ) {
        let dx = vector_subtract(p1, p0);
        let dy = vector_subtract(p2, p0);
        plane.normal = cross_product(dx, dy);
        VectorNormalize(&mut plane.normal);

        plane.dist = dot_product(p0, plane.normal);
        plane.r#type = plane_type_for_normal(plane.normal);
        set_plane_signbits(plane);

        // Raven's non-`_XBOX` arm: `side->plane = plane;`
        side.plane = plane as *mut cplane_t;
    }

    /// `CCMPatch::CreatePatchPlaneData` — builds the 2 collision brushes (5
    /// sides/planes each) for this patch's terxel, then smooths the shared
    /// edge with the x/y-adjacent patch's brush (`GetAdjacentBrushX/Y`,
    /// `:461,483`, live under the unconditional `_SMOOTH_TERXEL_BRUSH`
    /// `#define`, `cm_terrain.cpp:18`). `ls` is `&mut`: it both reads the
    /// landscape's terxel/coords/mins (`owner->Get…`) and writes into the
    /// shared brush arena (this patch's own slice, plus the adjacent patch's
    /// slice via `GetAdjacentBrushX/Y`, RMG-D7).
    ///
    /// Source: `oracle/codemp/qcommon/cm_terrain.cpp:302-522`
    pub fn create_patch_plane_data(&mut self, ls: &mut CmLandScape) {
        let terxels = ls.terxels;
        let num_brushes = (terxels * terxels * 2) as usize;
        self.num_brushes = num_brushes as i32;
        let real_width = ls.width + 1;

        let side_base = self.brush_offset + num_brushes * core::mem::size_of::<cbrush_t>();
        let plane_base = side_base
            + num_brushes * BRUSH_SIDES_PER_TERXEL * 2 * core::mem::size_of::<cbrushside_t>();

        // SAFETY: `arena_ptr` is a raw pointer into `ls`'s shared brush
        // arena (RMG-D7), captured once and reused for every disjoint-index
        // access to THIS patch's own terxel entries below — deliberately
        // untied from `ls`'s borrow so the y/x-adjacent-brush lookups
        // (`get_adjacent_brush_x/y`, which reborrow `ls`) can run alongside
        // it. This mirrors Raven's aliasing raw `cbrush_t*` pointer walk
        // over the same `Z_Malloc`'d buffer: every index touched here is
        // either this not-yet-built terxel (`brush_idx`/`side_idx`/
        // `plane_idx`) or an already-fully-built one reached only through
        // `get_adjacent_brush_x/y` — never both at once.
        let arena_ptr = ls.patch_brush_data.as_mut_ptr();

        let mut brush_idx = 0usize;
        let mut side_idx = 0usize;
        let mut plane_idx = 0usize;

        for y in self.hy..(self.hy + terxels) {
            for x in self.hx..(self.hx + terxels) {
                let offsets = terxel_corner_offsets(x, y, real_width);

                let mut local_coords = [[0.0f32; 3]; 8];
                for i in 0..4 {
                    let c = ls.coords[offsets[i] as usize];
                    local_coords[i] = c;
                    local_coords[i + 4] = c;
                    // Set z of base of brush to bottom of landscape brush.
                    local_coords[i + 4][2] = ls.bounds[0][2];
                }

                // Set the bounds of the terxel.
                let mut mins = [MAX_WORLD_COORD, MAX_WORLD_COORD, MAX_WORLD_COORD];
                let mut maxs = [MIN_WORLD_COORD, MIN_WORLD_COORD, MIN_WORLD_COORD];
                for corner in &local_coords {
                    for j in 0..3 {
                        if corner[j] < mins[j] {
                            mins[j] = corner[j];
                        }
                        if corner[j] > maxs[j] {
                            maxs[j] = corner[j];
                        }
                    }
                }
                for j in 0..3 {
                    mins[j] -= 1.0;
                    maxs[j] += 1.0;
                }

                // SAFETY: `brush_idx`/`brush_idx + 1` address this patch's
                // own not-yet-touched terxel brushes — disjoint from every
                // other index used this iteration.
                unsafe {
                    let b0 = brush_ptr(arena_ptr, self.brush_offset, brush_idx);
                    (*b0).bounds = [mins, maxs];
                    (*b0).contents = self.content_flags;
                    (*b0).numsides = 5;
                    (*b0).sides = side_ptr(arena_ptr, side_base, side_idx);

                    let b1 = brush_ptr(arena_ptr, self.brush_offset, brush_idx + 1);
                    (*b1).bounds = [mins, maxs];
                    (*b1).contents = self.content_flags;
                    (*b1).numsides = 5;
                    (*b1).sides = side_ptr(arena_ptr, side_base, side_idx + 8);
                }

                // Create the planes of the 2 triangles that make up the
                // tops of the brushes.
                self.init_plane_at(
                    arena_ptr,
                    side_base,
                    plane_base,
                    side_idx,
                    plane_idx,
                    local_coords[0],
                    local_coords[1],
                    local_coords[2],
                );
                self.init_plane_at(
                    arena_ptr,
                    side_base,
                    plane_base,
                    side_idx + 8,
                    plane_idx + 8,
                    local_coords[3],
                    local_coords[2],
                    local_coords[1],
                );

                // Create the bottom face of the brushes.
                self.init_plane_at(
                    arena_ptr,
                    side_base,
                    plane_base,
                    side_idx + 1,
                    plane_idx + 1,
                    local_coords[4],
                    local_coords[6],
                    local_coords[5],
                );
                self.init_plane_at(
                    arena_ptr,
                    side_base,
                    plane_base,
                    side_idx + 9,
                    plane_idx + 9,
                    local_coords[7],
                    local_coords[5],
                    local_coords[6],
                );

                // Create the 3 vertical faces.
                self.init_plane_at(
                    arena_ptr,
                    side_base,
                    plane_base,
                    side_idx + 2,
                    plane_idx + 2,
                    local_coords[0],
                    local_coords[2],
                    local_coords[4],
                );
                self.init_plane_at(
                    arena_ptr,
                    side_base,
                    plane_base,
                    side_idx + 10,
                    plane_idx + 10,
                    local_coords[3],
                    local_coords[1],
                    local_coords[7],
                );

                self.init_plane_at(
                    arena_ptr,
                    side_base,
                    plane_base,
                    side_idx + 3,
                    plane_idx + 3,
                    local_coords[0],
                    local_coords[4],
                    local_coords[1],
                );
                self.init_plane_at(
                    arena_ptr,
                    side_base,
                    plane_base,
                    side_idx + 11,
                    plane_idx + 11,
                    local_coords[3],
                    local_coords[7],
                    local_coords[2],
                );

                self.init_plane_at(
                    arena_ptr,
                    side_base,
                    plane_base,
                    side_idx + 4,
                    plane_idx + 4,
                    local_coords[2],
                    local_coords[1],
                    local_coords[6],
                );
                self.init_plane_at(
                    arena_ptr,
                    side_base,
                    plane_base,
                    side_idx + 12,
                    plane_idx + 12,
                    local_coords[5],
                    local_coords[1],
                    local_coords[6],
                );

                // SAFETY: `plane_idx + 8` was initialized just above.
                let v = unsafe {
                    let p8 = &*plane_ptr(arena_ptr, plane_base, plane_idx + 8);
                    dot_product(p8.normal, local_coords[0]) - p8.dist
                };

                if v < 0.0 {
                    // SAFETY: `b0`/`b1`'s current `numsides` addresses their
                    // own next free side/plane slot (reserved capacity,
                    // RMG-D7's arena sizing).
                    unsafe {
                        let ns0 =
                            (*brush_ptr(arena_ptr, self.brush_offset, brush_idx)).numsides as usize;
                        self.init_plane_at(
                            arena_ptr,
                            side_base,
                            plane_base,
                            side_idx + ns0,
                            plane_idx + ns0,
                            local_coords[3],
                            local_coords[2],
                            local_coords[1],
                        );
                        (*brush_ptr(arena_ptr, self.brush_offset, brush_idx)).numsides += 1;

                        let ns1 = (*brush_ptr(arena_ptr, self.brush_offset, brush_idx + 1)).numsides
                            as usize;
                        self.init_plane_at(
                            arena_ptr,
                            side_base,
                            plane_base,
                            side_idx + 8 + ns1,
                            plane_idx + 8 + ns1,
                            local_coords[0],
                            local_coords[1],
                            local_coords[2],
                        );
                        (*brush_ptr(arena_ptr, self.brush_offset, brush_idx + 1)).numsides += 1;
                    }
                }

                // Determine if we need to smooth the brush transition from
                // the brush above us.
                if y > 0 && (y as f32) < ls.patch_size[1] - 1.0 {
                    let cmp_coord = if (y + x) & 1 != 0 {
                        local_coords[2]
                    } else {
                        local_coords[1]
                    };
                    let above = self.get_adjacent_brush_y(ls, x, y);
                    let above_sides = above.sides;
                    // SAFETY: `above_sides` was written by an earlier `Init`
                    // (this patch's own prior terxel iteration, or an
                    // already fully-constructed neighboring patch), so it
                    // points at a live `cbrushside_t` with a live `plane`.
                    let (above_normal, above_dist) = unsafe {
                        let above_plane = (*above_sides).plane;
                        ((*above_plane).normal, (*above_plane).dist)
                    };
                    let v = dot_product(above_normal, cmp_coord) - above_dist;

                    if v < 0.0 {
                        // SAFETY: `above` remains borrowed from `ls` for
                        // this whole block; `arena_ptr` addresses this
                        // patch's own (disjoint) terxel entry.
                        unsafe {
                            let b0 = brush_ptr(arena_ptr, self.brush_offset, brush_idx);
                            let ns0 = (*b0).numsides as usize;
                            let dst = side_ptr(arena_ptr, side_base, side_idx + ns0);
                            copy_side(dst, above_sides);
                            (*b0).numsides += 1;

                            let ns_above = above.numsides as usize;
                            let dst_above = above_sides.add(ns_above);
                            let src_local = side_ptr(arena_ptr, side_base, side_idx);
                            copy_side(dst_above, src_local);
                            above.numsides += 1;
                        }
                    }
                }

                // Determine if we need to smooth the brush transition from
                // the brush to the left of us.
                if x > 0 && (x as f32) < ls.patch_size[0] - 1.0 {
                    let above = self.get_adjacent_brush_x(ls, x, y);
                    let above_sides = above.sides;
                    // SAFETY: see the y-adjacent block above.
                    let (above_normal, above_dist) = unsafe {
                        let above_plane = (*above_sides).plane;
                        ((*above_plane).normal, (*above_plane).dist)
                    };
                    let v = dot_product(above_normal, local_coords[1]) - above_dist;

                    if v < 0.0 {
                        // SAFETY: see the y-adjacent block above.
                        unsafe {
                            if (x + y) & 1 != 0 {
                                let b0 = brush_ptr(arena_ptr, self.brush_offset, brush_idx);
                                let ns0 = (*b0).numsides as usize;
                                let dst = side_ptr(arena_ptr, side_base, side_idx + ns0);
                                copy_side(dst, above_sides);
                                (*b0).numsides += 1;

                                let ns_above = above.numsides as usize;
                                let dst_above = above_sides.add(ns_above);
                                let src_local = side_ptr(arena_ptr, side_base, side_idx);
                                copy_side(dst_above, src_local);
                                above.numsides += 1;
                            } else {
                                let b1 = brush_ptr(arena_ptr, self.brush_offset, brush_idx + 1);
                                let ns1 = (*b1).numsides as usize;
                                let dst = side_ptr(arena_ptr, side_base, side_idx + 8 + ns1);
                                copy_side(dst, above_sides);
                                (*b1).numsides += 1;

                                let ns_above = above.numsides as usize;
                                let dst_above = above_sides.add(ns_above);
                                let src_local = side_ptr(arena_ptr, side_base, side_idx + 8);
                                copy_side(dst_above, src_local);
                                above.numsides += 1;
                            }
                        }
                    }
                }

                // Increment to next terxel.
                brush_idx += 2;
                side_idx += 16;
                plane_idx += 16;
            }
        }
    }

    /// Private helper: builds a `cbrushside_t`/`cplane_t` pair at the given
    /// arena indices via [`CmPatch::init_plane`]. Factored out of
    /// [`CmPatch::create_patch_plane_data`] purely to avoid repeating the
    /// raw-pointer-dereference boilerplate at each of that method's 8+
    /// `InitPlane` call sites — not a Raven method.
    #[allow(clippy::too_many_arguments)]
    fn init_plane_at(
        &self,
        arena_ptr: *mut u8,
        side_base: usize,
        plane_base: usize,
        side_index: usize,
        plane_index: usize,
        p0: vec3_t,
        p1: vec3_t,
        p2: vec3_t,
    ) {
        // SAFETY: callers only ever pass indices that address this patch's
        // own reserved (RMG-D7-sized) side/plane slots, one at a time, with
        // no other live reference to the same slot.
        unsafe {
            let side = &mut *side_ptr(arena_ptr, side_base, side_index);
            let plane = &mut *plane_ptr(arena_ptr, plane_base, plane_index);
            self.init_plane(side, plane, p0, p1, p2);
        }
    }

    /// `CCMPatch::GetAdjacentBrushX` — the brush directly x-adjacent to
    /// terxel `(x, y)`, walking into the neighboring patch (via
    /// `ls.get_patch(...)`, RMG-D4h's threaded-owner substitute for
    /// `owner->GetPatch(...)`) when the terxel crosses a patch boundary.
    /// Returns `&mut` because `CreatePatchPlaneData`'s live caller mutates
    /// the returned brush's `sides`/`numsides` (`cm_terrain.cpp:497-509`).
    /// RMG-D7/§B5 picked the shared-arena shape precisely because this
    /// sibling-walking read exists.
    ///
    /// Source: `oracle/codemp/qcommon/cm_terrain.cpp:272-300`
    pub fn get_adjacent_brush_x<'a>(
        &self,
        ls: &'a mut CmLandScape,
        x: i32,
        y: i32,
    ) -> &'a mut cbrush_t {
        let (other_patch, index) = adjacent_brush_x_index(ls.terxels, x, y);
        let brush_offset = match other_patch {
            Some((px, py)) => ls.get_patch(px, py).brush_offset,
            None => self.brush_offset,
        };
        // SAFETY: `index` addresses an already-`Init`'d terxel's brush entry
        // — either this patch's own prior-iteration entry or a fully
        // constructed neighboring patch's (both built earlier in
        // `UpdatePatches`' scan order), never the terxel currently being
        // built by the live caller (`CreatePatchPlaneData`).
        unsafe { &mut *brush_ptr(ls.patch_brush_data.as_mut_ptr(), brush_offset, index) }
    }

    /// `CCMPatch::GetAdjacentBrushY` — the brush directly y-adjacent to
    /// terxel `(x, y)`; see [`get_adjacent_brush_x`](Self::get_adjacent_brush_x)
    /// for the shape rationale (sibling patch walk via `ls`, `&mut` return).
    ///
    /// Source: `oracle/codemp/qcommon/cm_terrain.cpp:246-270`
    pub fn get_adjacent_brush_y<'a>(
        &self,
        ls: &'a mut CmLandScape,
        x: i32,
        y: i32,
    ) -> &'a mut cbrush_t {
        let (other_patch, index) = adjacent_brush_y_index(ls.terxels, x, y);
        let brush_offset = match other_patch {
            Some((px, py)) => ls.get_patch(px, py).brush_offset,
            None => self.brush_offset,
        };
        // SAFETY: see `get_adjacent_brush_x`.
        unsafe { &mut *brush_ptr(ls.patch_brush_data.as_mut_ptr(), brush_offset, index) }
    }

    /// Raven `CCMPatch::GetCollisionData` — the `cbrush_s *mPatchBrushData` base
    /// of this patch's slice. Bridges the C-track `CM_HandlePatchCollision`
    /// brush walk to the RMG-D7 shared arena (ruling 46): `mPatchBrushData`
    /// becomes `ls.patch_brush_data + brush_offset`, mirroring [`brush_ptr`].
    /// Source: `oracle/codemp/qcommon/cm_landscape.h:117`
    pub fn get_collision_data(&self, ls: &mut CmLandScape) -> *mut cbrush_t {
        // SAFETY: `brush_offset` is this patch's arena slice start (§ Init);
        // the arena outlives the returned base for the collision walk.
        unsafe { brush_ptr(ls.patch_brush_data.as_mut_ptr(), self.brush_offset, 0) }
    }

    /// Raven `CCMPatch::GetNumBrushes` — `mNumBrushes` (= `terxels*terxels*2`,
    /// set in `create_patch_plane_data`).
    /// Source: `oracle/codemp/qcommon/cm_landscape.h:116`
    pub const fn get_num_brushes(&self) -> i32 {
        self.num_brushes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- PlaneTypeForNormal / SetPlaneSignbits (cm_terrain.cpp:230-232) --

    #[test]
    fn plane_type_for_normal_axial() {
        assert_eq!(plane_type_for_normal([1.0, 0.0, 0.0]), PLANE_X as u8);
        assert_eq!(plane_type_for_normal([0.0, 1.0, 0.0]), PLANE_Y as u8);
        assert_eq!(plane_type_for_normal([0.0, 0.0, 1.0]), PLANE_Z as u8);
    }

    #[test]
    fn plane_type_for_normal_non_axial() {
        // Raven's macro checks `x[0]==1.0` before `x[1]==1.0` before
        // `x[2]==1.0` — a normal that is none of those exactly is
        // PLANE_NON_AXIAL even if one axis dominates.
        assert_eq!(
            plane_type_for_normal([0.7071, 0.7071, 0.0]),
            PLANE_NON_AXIAL as u8
        );
    }

    #[test]
    fn set_plane_signbits_matches_negative_axes() {
        let mut plane = cplane_t {
            normal: [-1.0, 2.0, -3.0],
            dist: 0.0,
            r#type: 0,
            signbits: 0,
            pad: [0, 0],
        };
        set_plane_signbits(&mut plane);
        // bit0 (x<0) | bit2 (z<0) = 0b101 = 5
        assert_eq!(plane.signbits, 0b101);
    }

    // -- InitPlane (cm_terrain.cpp:223-241): a unit right-triangle in the
    //    XY plane should produce a +Z-facing, axial plane. --

    #[test]
    fn init_plane_builds_expected_plane() {
        let patch = CmPatch::default();
        let mut side = cbrushside_t {
            plane: core::ptr::null_mut(),
            shaderNum: 0,
        };
        let mut plane = cplane_t {
            normal: [0.0, 0.0, 0.0],
            dist: 0.0,
            r#type: 0,
            signbits: 0,
            pad: [0, 0],
        };
        patch.init_plane(
            &mut side,
            &mut plane,
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        );
        assert!((plane.normal[2] - 1.0).abs() < 1e-5);
        assert_eq!(plane.r#type, PLANE_Z as u8);
        assert_eq!(plane.signbits, 0);
        assert_eq!(side.plane, &mut plane as *mut cplane_t);
    }

    // -- adjacent_brush_{x,y}_index (cm_terrain.cpp:246-300): same-patch vs
    //    cross-patch branch + index parity. --

    #[test]
    fn adjacent_brush_y_same_patch_when_not_crossing_boundary() {
        // terxels=4; y=5 -> yo1=1, y-1=4 -> yo2=0; yo2(0) > yo1(1) is false
        // => stays in `self` (Raven's `patch = this;`).
        let (other, index) = adjacent_brush_y_index(4, 2, 5);
        assert_eq!(other, None);
        // yo2=0, xo=2%4=2 -> index = (0*4+2)*2+1 = 5
        assert_eq!(index, 5);
    }

    #[test]
    fn adjacent_brush_y_crosses_into_other_patch_at_boundary() {
        // terxels=4; y=4 -> yo1=0, y-1=3 -> yo2=3; yo2(3) > yo1(0) is true
        // => crosses into the patch above (Raven's owner->GetPatch branch).
        let (other, index) = adjacent_brush_y_index(4, 2, 4);
        assert_eq!(other, Some((0, 0))); // x/terxels=0, (y-1)/terxels=0
        assert_eq!(index, (3 * 4 + 2) * 2 + 1);
    }

    #[test]
    fn adjacent_brush_x_parity_selects_brush_slot() {
        let terxels = 4;
        // (x+y) even -> +1 (second brush of the terxel pair).
        let (_, idx_even) = adjacent_brush_x_index(terxels, 2, 2);
        assert_eq!(idx_even % 2, 1);
        // (x+y) odd -> no +1 (first brush of the pair).
        let (_, idx_odd) = adjacent_brush_x_index(terxels, 3, 2);
        assert_eq!(idx_odd % 2, 0);
    }

    // -- terxel_corner_offsets (cm_terrain.cpp:325-341): checkerboard split. --

    #[test]
    fn terxel_corner_offsets_odd_diagonal() {
        // (x+y) odd: [0]=TL,[1]=TR,[2]=BL,[3]=BR directly.
        let o = terxel_corner_offsets(1, 0, 10); // x+y=1, odd
        assert_eq!(o, [0 * 10 + 1, 0 * 10 + 2, 1 * 10 + 1, 1 * 10 + 2]);
    }

    #[test]
    fn terxel_corner_offsets_even_diagonal_is_permuted() {
        // (x+y) even: [0]=TR,[1]=BR,[2]=TL,[3]=BL.
        let o = terxel_corner_offsets(0, 0, 10); // x+y=0, even
        let tl = 0;
        let tr = 1;
        let bl = 10;
        let br = 11;
        assert_eq!(o, [tr, br, tl, bl]);
    }

    // -- brush_region_len (cm_terrain.cpp:212-215): matches the ctor's own
    //    per-block `size` formula. --

    #[test]
    fn brush_region_len_matches_ctor_formula() {
        let terxels = 4;
        let num_brushes_per_patch = terxels * terxels * 2;
        let expected = (num_brushes_per_patch as usize) * core::mem::size_of::<cbrush_t>()
            + (num_brushes_per_patch as usize)
                * BRUSH_SIDES_PER_TERXEL
                * 2
                * (core::mem::size_of::<cbrushside_t>() + core::mem::size_of::<cplane_t>());
        assert_eq!(brush_region_len(terxels), expected);
    }
}
