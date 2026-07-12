#![allow(non_camel_case_types, non_snake_case)]

//! `G2_Misc` internal — the assorted `G2_misc.cpp` free functions: the
//! model/bone debug listers, the anim-filename getter, the model-pointer
//! validity re-derivation every other file's wrappers open with, the
//! collision-trace + gore-apply-transform families, the world/inverse-matrix
//! math, the index-based surface locator, and the save/load (de)serializers.
//!
//! Per `docs/subsystems/ghoul2-server.md` roster (`misc.rs`, class "G2_Misc
//! internal"): `G2_TraceModels`/`TransformModel`/`GenerateWorldMatrix`/
//! `TransformPoint`/`TransformAndTranslatePoint`/`Inverse_Matrix`/
//! `FindSurface`, `G2_SetupModelPointers`, `G2_SaveGhoul2Models`/
//! `LoadGhoul2Model`, list/name helpers. That one-liner's twelve public names
//! match `G2_local.h`'s own `// misc functions G2_misc.cpp` header block
//! (`:64-77`) exactly: `G2_List_Model_Surfaces`, `G2_List_Model_Bones`,
//! `G2_GetAnimFileName`, `G2_TraceModels`, `TransformAndTranslatePoint`,
//! `G2_TransformModel`, `G2_GenerateWorldMatrix`, `TransformPoint`,
//! `Inverse_Matrix`, `G2_FindSurface`, `G2_SaveGhoul2Models`,
//! `G2_LoadGhoul2Model` — plus `G2_SetupModelPointers`, called out separately
//! in the doc's Method transcription table. Enumerating the oracle TU
//! (`oracle/codemp/ghoul2/G2_misc.cpp`) directly per porting-rules
//! §F17/CLAUDE.md ("private helpers included") turns up the private helper
//! chains those twelve need to be complete — `G2_DecideTraceLod`,
//! `G2_TraceSurfaces`/`G2_TracePolys`/`G2_RadiusTracePolys` (+ the geometry
//! primitives `G2_AreaOfTri`/`G2_BuildHitPointST`/`G2_SegmentTriangleTest`,
//! shared with `gore/gore_set.rs`'s `G2_GorePolys`) for the trace family, and
//! `G2_TransformSurfaces`/`R_TransformEachSurface`/`Create_Matrix` for the
//! transform/matrix family — all stubbed below alongside the twelve.
//!
//! **`_G2_GORE` ON (`G2SV-D5` build config).** `G2_TraceModels`/
//! `G2_TransformModel` each have two `#ifdef _G2_GORE`/`#else` declarations in
//! the oracle (`G2_local.h:69-77`, `G2_misc.cpp:1514-1517,559-563`); only the
//! `_G2_GORE`-on arm (the wider parameter list) ever compiles in this build,
//! so there is exactly one signature to transcribe per name, not two.
//!
//! **Doc/oracle gaps found while enumerating this class (reported to the
//! caller, not fixed here):**
//! 1. The doc's Method transcription table cites `G2_SetupModelPointers` at
//!    `G2_misc.cpp:1839` — that line is only the two forward *declarations*
//!    (`qboolean G2_SetupModelPointers(CGhoul2Info *ghlInfo);` /
//!    `(CGhoul2Info_v &ghoul2);`, matching this TU's own late in-file
//!    declaration-before-use idiom). The actual *definitions* live in
//!    `G2_API.cpp:2675-2693` (single-instance overload) and `:2773-2783`
//!    (vector overload). The roster assignment to `misc.rs` is followed as
//!    written (LAW); only the pinpoint citation is wrong.
//! 2. Three more functions are physically defined in `G2_misc.cpp` — the file
//!    this roster row's summary and the Method transcription table both cite
//!    — that neither names: `G2_LerpAngles` (`:1912-1936`), `G2_FreeSaveBuffer`
//!    (`:1812-1815`), and `G2_FindConfigStringSpace` (`:1817-1836`). None of
//!    the three is declared in any header (`grep` across
//!    `oracle/codemp/ghoul2/*.h` — nothing), and none has a caller anywhere in
//!    `oracle/codemp/` (`G2API_FreeSaveBuffer`, `G2_API.cpp:2482-2485`, does
//!    its own bare `Z_Free` rather than calling `G2_FreeSaveBuffer`). This is
//!    the same zero-caller shape the doc's own divergences list already
//!    applies to `G2API_AddSkinGore`/`ResetGoreTag`/`G2_GetGoreRecord` (§20
//!    drop), but the doc never lists these three. Treated identically here —
//!    dropped with this module-doc note, not stubbed — and reported upstream
//!    as a doc completeness gap rather than silently absorbed.
//! 3. `api_surfaces.rs`'s `g2api_get_surface_name`/`g2api_list_surfaces` doc
//!    comments cite the index-based `G2_FindSurface(void*, int, int)` as
//!    living in "`surfaces.rs`". Both this roster row (`FindSurface` named
//!    explicitly for `misc.rs`) and `surfaces.rs`'s own module doc comment
//!    (which correctly distinguishes this overload from its *name-based*
//!    `G2_FindSurface(CGhoul2Info*, surfaceInfo_v&, const char*, int*)` sibling
//!    and assigns the index-based one here) agree it belongs in `misc.rs`.
//!    Cross-file inconsistency in `api_surfaces.rs`, reported upstream, not
//!    fixed here (out of this task's file scope).
//! 4. `CMiniHeap *G2VertSpace` (the scratch vertex-transform-space allocator
//!    `G2_TransformModel`/`G2_TransformSurfaces`/`R_TransformEachSurface` all
//!    take) has no doc-pinned Rust shape anywhere in `## Seam definition` or
//!    `## State ownership`. `api_collision.rs` already reports this exact gap
//!    upstream for `g2api_collision_detect`'s dropped `G2VertSpace` parameter;
//!    followed here for consistency (same missing-service class, not a new
//!    decision) rather than re-reported as a second finding.
//! 5. **(new) `G2_GorePolys` reachability claim vs. oracle ground truth.** The
//!    doc's "Gore store" section and `GoreState.gore_touch`'s state-ownership
//!    row both assert "`GoreTouch++` runs server-side on every trace" because
//!    `G2_GorePolys` (`G2_misc.cpp:804`) is "reached from the in-scope
//!    collision path `G2_TraceModels`". Reading the sole call site
//!    (`G2_misc.cpp:1494`, inside `G2_TraceSurfaces`'s `if (TS.collRecMap)
//!    {...trace...} else { G2_GorePolys(...); }`) shows `G2_GorePolys` fires
//!    only on the **`else`** arm — i.e. only when `TS.collRecMap` (== the
//!    caller's `collRecMap` argument) is **null**. The sole real server caller
//!    (`G2API_CollisionDetect`/`CollisionDetectCache`, `api_collision.rs`)
//!    always passes the address of a real stack array, never null; the only
//!    caller that ever passes a literal `0` for `collRecMap` is the
//!    graph-dead `G2API_AddSkinGore` (`G2_API.cpp:2601`: `G2_TraceModels(...,
//!    0, gore.entNum, 0, lod, 0.0f, ..., &gore, qtrue);`). So `G2_GorePolys`
//!    (and its `GoreTouch++`) is **not** reached by any live server path
//!    either — the doc's ground-truth claim is wrong, not just this file's
//!    citation. Because this port's `g2_trace_models` signature (frozen by
//!    the already-landed `api_collision.rs` call site, which always supplies
//!    a real `&mut [CollisionRecord_t]`, never an `Option`) makes the null
//!    branch structurally unreachable, the `if (TS.collRecMap) {...} else {
//!    G2_GorePolys(...) }` guard folds permanently to its true arm here
//!    (§C10) and `g2_trace_surfaces` never calls into `gore/gore_set.rs` at
//!    all. Reported upstream, not fixed here (fixing the doc's prose is out
//!    of this file's scope).
//! 6. **(new) `g2_trace_models` has no way to receive `G2_GenerateWorldMatrix`'s
//!    `worldMatrix`.** The oracle's `G2_TracePolys`/`G2_RadiusTracePolys`
//!    transform each hit point/normal back into WORLD space via the file-scope
//!    `worldMatrix` (`G2_misc.cpp:1137,1140,1375,1409`), which
//!    `G2_GenerateWorldMatrix` derives from `(angles, origin)`. Neither this
//!    file's already-declared `g2_trace_models` signature nor its sole caller
//!    (`api_collision.rs`'s `g2api_collision_detect`/`_cache`, which computes
//!    `let (_world_matrix, world_matrix_inv) = g2_generate_world_matrix(...)`
//!    and explicitly **discards** the forward matrix) threads that matrix
//!    through. This function therefore uses the **identity** matrix as the
//!    best available substitute — exact when `angles == (0,0,0)` and
//!    `origin == (0,0,0)` (the untranslated/unrotated case), silently wrong
//!    otherwise. Reported upstream; fixing it needs a signature change to
//!    `g2_trace_models` (out of this file's authority alone, since
//!    `api_collision.rs` is a sibling porter's file) or a `Ghoul2System`-level
//!    per-construct scratch slot for it.
//! 7. **(new) the per-surface `mTransformedVertsArray` pointer-in-`int` design
//!    has no memory-safe Rust equivalent.** Raven's `CGhoul2Info::
//!    mTransformedVertsArray` is an `int[numSurfaces]` whose entries are
//!    `(int)TransformedVerts` — a per-surface `float*` from `CMiniHeap`,
//!    reinterpreted as an `int` so it fits the declared element type
//!    (`G2_misc.cpp:417,639-646`); the trace family reads it back via
//!    `(float *)TS.TransformedVertsArray[surface->thisSurfaceIndex]`
//!    (`:843,887,1098,1267`). The already-frozen `CGhoul2Info.
//!    transformed_verts_array: Option<Vec<i32>>` (`shared/cghoul2_info.rs`,
//!    not this file's to edit) preserves the *shape* (an owned `i32` buffer)
//!    but a real pointer cannot be losslessly stored in an `i32` on a 64-bit
//!    host. This file's `g2_transform_model`/`g2_trace_models` instead treat
//!    the buffer as **one flat, bit-punned `f32`→`i32` array per model**,
//!    populated by a depth-first walk of the surface tree
//!    (`g2_transform_surfaces`, appending each visited surface's `numVerts*5`
//!    floats in order) and consumed by the **structurally identical**
//!    depth-first walk (`g2_trace_surfaces`, advancing a `verts_cursor` by the
//!    same `numVerts*5` stride per visited surface). Because both walks visit
//!    the same model's surface tree in the same deterministic order within
//!    one collision-detect call, this reproduces the oracle's *observable*
//!    per-surface vertex data exactly (porting-rules §A1: internals free,
//!    seam preserved) without ever encoding a pointer as an `i32`. Reported
//!    upstream as a genuine architecture gap in the frozen sibling type, not
//!    fixed there.
//! 8. **(new) `api_models.rs::g2_test_model_pointers` omits setting
//!    `ghl_info.a_header`.** The oracle's `G2_TestModelPointers`
//!    (`G2_API.cpp:2606-2663`) sets `ghlInfo->aHeader = ghlInfo->animModel->
//!    mdxa;` (`:2739`) alongside `animModel`; `api_models.rs`'s copy resolves
//!    `animModel` but never writes the parallel `a_header` field. This file's
//!    own `g2_setup_model_pointers` (the sibling overload for the
//!    already-initialized-model call sites) sets both, matching the oracle.
//!    Reported upstream; not fixed here (out of this file's scope).
//!
//! **Register-model host gap (not a new finding — same class `api_models.rs`
//! already reports).** `G2_List_Model_Surfaces`/`G2_List_Model_Bones`/
//! `G2_GetAnimFileName`/`G2_SetupModelPointers` all call `RE_RegisterModel`/
//! `RE_RegisterServerModel` (a filename → `qhandle_t` register), which has no
//! `EngineHost` method (`## Seam definition`'s 15 methods only resolve an
//! *already-registered* handle to its parsed block). This file duplicates
//! `api_models.rs`'s `register_model`/`register_server_model`/
//! `g2_should_register_server` divergence-via-`host.error` helpers (no shared
//! home exists yet for them, same as that file's own `GHOUL2_ZONETRANSALLOC`/
//! `MAX_G2_MODELS` duplication precedent) rather than inventing a fake handle.

use core::ffi::c_void;

use mp_host_interface::EngineHost;
use mp_qshared::shared::{errorParm_t, mdxaBone_t, qhandle_t, vec3_t, CollisionRecord_t};

use crate::ghoul2_system::{BoneCacheId, Ghoul2System};
use crate::gore::sskin_gore_data::SSkinGoreData;
use crate::render::bone_cache::eval_bone_cache;
use crate::shared::bolt_info_t::boltInfo_t;
use crate::shared::bone_info_t::boneInfo_t;
use crate::shared::cghoul2_info::CGhoul2Info;
use crate::shared::cghoul2_info_v::CGhoul2Info_v;
use crate::shared::eg2_collision::EG2_Collision;
use crate::shared::surface_info_t::surfaceInfo_t;

// ---------------------------------------------------------------------------
// Small `#define` constants this file reads, duplicated locally per the
// established no-shared-home-yet convention (`api_collision.rs`'s
// `GHOUL2_ZONETRANSALLOC`, `api_models.rs`'s `GHOUL2_NEWORIGIN`, ...).
// ---------------------------------------------------------------------------

/// Raven `#define GHOUL2_NOCOLLIDE 0x001`.
/// Source: `oracle/codemp/ghoul2/ghoul2_shared.h:229`
const GHOUL2_NOCOLLIDE: i32 = 0x001;

/// Raven `#define G2SURFACEFLAG_NODESCENDANTS 0x00000100`.
/// Source: `oracle/codemp/renderer/mdx_format.h:49`
const G2SURFACEFLAG_NODESCENDANTS: u32 = 0x00000100;

/// Raven `#define G2_FRONTFACE 1`.
/// Source: `oracle/codemp/ghoul2/ghoul2_shared.h:459`
const G2_FRONTFACE: i32 = 1;
/// Raven `#define G2_BACKFACE 0`.
/// Source: `oracle/codemp/ghoul2/ghoul2_shared.h:460`
const G2_BACKFACE: i32 = 0;

// ---------------------------------------------------------------------------
// Raw mdxm/mdxa header/record field offsets (`G2SV-D5`: the header types
// themselves are never named in this crate; duplicated per-file per the
// `api_models.rs`/`api_collision.rs`/`api_surfaces.rs` precedent — no shared
// home exists yet). Every field in `oracle/codemp/renderer/mdx_format.h` is a
// 4-byte-aligned int/float/`char[64]`, so natural alignment introduces no
// padding.
// ---------------------------------------------------------------------------

const MAX_QPATH: usize = 64;

// mdxmHeader_t (mdx_format.h:151-172).
const MDXM_OFS_ANIM_INDEX: usize = 136;
const MDXM_OFS_NUM_LODS: usize = 144;
const MDXM_OFS_OFS_LODS: usize = 148;
const MDXM_OFS_NUM_SURFACES: usize = 152;
const MDXM_OFS_OFS_SURF_HIERARCHY: usize = 156;
const MDXM_OFS_OFS_END: usize = 160;
/// `sizeof(mdxmHeader_t)` — `ofsEnd`(160) + 4. Several oracle call sites walk
/// straight to `header + sizeof(mdxmHeader_t)` instead of re-reading
/// `ofsSurfHierarchy` (they are always the same location; kept literal here).
const MDXM_HEADER_SIZE: usize = 164;
const MDXM_OFS_ANIM_NAME: usize = 72;

// mdxaHeader_t (mdx_format.h:344-364).
const MDXA_OFS_NUM_BONES: usize = 84;
const MDXA_OFS_OFS_END: usize = 96;
/// `sizeof(mdxaHeader_t)` — `ofsEnd`(96) + 4.
const MDXA_HEADER_SIZE: usize = 100;

// mdxmSurfHierarchy_t (mdx_format.h:190-196).
const SURF_HIER_OFS_NAME: usize = 0;
const SURF_HIER_OFS_FLAGS: usize = 64;
const SURF_HIER_OFS_NUM_CHILDREN: usize = 140;
const SURF_HIER_OFS_CHILD_INDEXES: usize = 144;

// mdxmLOD_t (mdx_format.h:210-215): `{ int ofsEnd; }`.
const MDXM_LOD_OFS_END: usize = 0;
const MDXM_LOD_SIZE: usize = 4;

// mdxmSurface_t (mdx_format.h:230-245).
const MDXM_SURF_OFS_THIS_SURFACE_INDEX: usize = 4;
const MDXM_SURF_OFS_NUM_VERTS: usize = 12;
const MDXM_SURF_OFS_OFS_VERTS: usize = 16;
const MDXM_SURF_OFS_NUM_TRIANGLES: usize = 20;
const MDXM_SURF_OFS_OFS_TRIANGLES: usize = 24;
const MDXM_SURF_OFS_OFS_BONE_REFS: usize = 32;

// mdxmTriangle_t `{ int indexes[3]; }`.
const MDXM_TRIANGLE_SIZE: usize = 12;

// mdxmVertex_t (32 bytes: normal(12) + vertCoords(12) + packed(4) + BoneWeightings(4)).
const MDXM_VERTEX_SIZE: usize = 32;

// mdxaSkelOffsets_t / mdxmHierarchyOffsets_t / mdxmLODSurfOffset_t: `int
// offsets[N]`, each entry 4 bytes.
const OFFSETS_ENTRY_SIZE: usize = 4;

// mdxaSkel_t (mdx_format.h:326-334): name[64](0) + flags(64) + parent(68) +
// BasePoseMat(72,48) + BasePoseMatInv(120,48) + numChildren(168) + children(172).
const MDXA_SKEL_OFS_NAME: usize = 0;
const MDXA_SKEL_OFS_BASEPOSE: usize = 72;
const MDXA_SKEL_OFS_NUM_CHILDREN: usize = 168;

/// Raven `#define GHOUL2_ZONETRANSALLOC 0x2000` — duplicated from
/// `api_collision.rs` per that file's own no-shared-home precedent.
/// Source: `oracle/codemp/ghoul2/ghoul2_shared.h:235`
#[allow(dead_code)]
const GHOUL2_ZONETRANSALLOC: i32 = 0x2000;

// ---------------------------------------------------------------------------
// Raw byte-arithmetic helpers over `EngineHost::model_mdxm`/`model_mdxa`
// blocks (`G2SV-D5`: never a named header type, only pointer + offset).
// ---------------------------------------------------------------------------

/// # Safety
/// `base` must be non-null and `offset..offset+4` must lie inside the block.
unsafe fn read_i32(base: *const c_void, offset: usize) -> i32 {
    unsafe {
        (base as *const u8)
            .add(offset)
            .cast::<i32>()
            .read_unaligned()
    }
}

/// # Safety
/// `base` must be non-null and `offset..offset+4` must lie inside the block.
unsafe fn read_u32(base: *const c_void, offset: usize) -> u32 {
    unsafe {
        (base as *const u8)
            .add(offset)
            .cast::<u32>()
            .read_unaligned()
    }
}

/// # Safety
/// `base` must be non-null and `offset..offset+4` must lie inside the block.
unsafe fn read_f32(base: *const c_void, offset: usize) -> f32 {
    unsafe {
        (base as *const u8)
            .add(offset)
            .cast::<f32>()
            .read_unaligned()
    }
}

/// # Safety
/// `base` must be non-null and `offset..offset+max_len` must lie inside the block.
unsafe fn read_cstr(base: *const c_void, offset: usize, max_len: usize) -> String {
    unsafe {
        let bytes = core::slice::from_raw_parts((base as *const u8).add(offset), max_len);
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(max_len);
        String::from_utf8_lossy(&bytes[..end]).into_owned()
    }
}

fn byte_add(base: *const c_void, offset: usize) -> *const c_void {
    // SAFETY: pure pointer arithmetic; the result is only ever dereferenced by
    // the `read_*` helpers above, whose own safety contract applies there.
    unsafe { (base as *const u8).add(offset) as *const c_void }
}

/// Read one flat, bit-punned vertex slot (module-doc gap note #7): 5 `f32`s
/// stored as `i32` bit patterns at `(base + idx*5)` in the model's
/// `transformed_verts_array`.
///
/// # Safety
/// `ptr` must be non-null and `base + idx*5 .. +5` must be in bounds.
unsafe fn read_flat_vert(ptr: *const i32, base: usize, idx: usize) -> [f32; 5] {
    unsafe {
        let p = ptr.add(base + idx * 5);
        [
            f32::from_bits(*p as u32),
            f32::from_bits(*p.add(1) as u32),
            f32::from_bits(*p.add(2) as u32),
            f32::from_bits(*p.add(3) as u32),
            f32::from_bits(*p.add(4) as u32),
        ]
    }
}

// ---------------------------------------------------------------------------
// `RE_RegisterModel`/`RE_RegisterServerModel`/`G2_ShouldRegisterServer` — no
// `EngineHost` equivalent (module-doc gap note); duplicated from
// `api_models.rs`'s identically-shaped helpers per that file's own
// no-shared-home precedent.
// ---------------------------------------------------------------------------

/// Raven `qboolean G2_ShouldRegisterServer(void)` gate, duplicated from
/// `api_models.rs::g2_should_register_server` (see that file's module doc
/// comment for the full gap analysis).
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:568-583`
fn g2_should_register_server(host: &mut impl EngineHost) -> bool {
    if host.cvar_integer("cl_running") != 0 {
        host.error(
            errorParm_t::ERR_DROP,
            "G2_ShouldRegisterServer: cl_running set in a DEDICATED build \u{2014} \
             Com_TheHunkMarkHasBeenMade/ShaderHashTableExists have no EngineHost service",
        );
    }
    true
}

/// Raven `RE_RegisterServerModel( fileName )` through the
/// `EngineHost::model_register` seam (the former ghoul2-server.md gap, closed
/// by user ruling 2026-07-12).
/// Source: `oracle/codemp/renderer/tr_model.cpp:588`
fn register_server_model(host: &mut impl EngineHost, file_name: &str) -> qhandle_t {
    host.model_register(file_name)
}

/// `RE_RegisterModel`'s client-path twin of [`register_server_model`]; same
/// gap, same divergence treatment.
fn register_model(host: &mut impl EngineHost, file_name: &str) -> qhandle_t {
    host.error(
        errorParm_t::ERR_DROP,
        &format!(
            "G2_Misc internal: EngineHost has no RE_RegisterModel(\"{file_name}\") equivalent \
             yet (docs/subsystems/ghoul2-server.md gap note, G2_API.cpp:282,312,359,2714)"
        ),
    )
}

// ---------------------------------------------------------------------------
// Vector math (`q_math.c`) — no crate-wide home exists yet (`native_math`
// only carries type aliases), so this file carries its own minimal set,
// matching the values Raven's `DotProduct`/`CrossProduct`/... macros compute.
// ---------------------------------------------------------------------------

fn v_dot(a: vec3_t, b: vec3_t) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn v_cross(a: vec3_t, b: vec3_t) -> vec3_t {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn v_sub(a: vec3_t, b: vec3_t) -> vec3_t {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn v_add(a: vec3_t, b: vec3_t) -> vec3_t {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn v_scale(a: vec3_t, s: f32) -> vec3_t {
    [a[0] * s, a[1] * s, a[2] * s]
}
fn v_length_squared(a: vec3_t) -> f32 {
    v_dot(a, a)
}
fn v_length(a: vec3_t) -> f32 {
    v_length_squared(a).sqrt()
}
fn v_normalize(a: &mut vec3_t) -> f32 {
    let len = v_length(*a);
    if len != 0.0 {
        let inv = 1.0 / len;
        a[0] *= inv;
        a[1] *= inv;
        a[2] *= inv;
    }
    len
}
/// The first 3 elements of a `mdxaBone_t` matrix row (Raven's `DotProduct`
/// macro only ever reads indices `[0][1][2]` regardless of the row's real
/// declared width).
fn row3(row: [f32; 4]) -> vec3_t {
    [row[0], row[1], row[2]]
}

/// Raven `AngleVectors` (`q_math.c:1315-1347`).
fn angle_vectors(angles: vec3_t) -> (vec3_t, vec3_t, vec3_t) {
    const DEG2RAD: f32 = core::f32::consts::PI * 2.0 / 360.0;
    // Raven indices: `angles[YAW]`=1, `angles[PITCH]`=0, `angles[ROLL]`=2.
    let angle_yaw = angles[1] * DEG2RAD;
    let (sy, cy) = (angle_yaw.sin(), angle_yaw.cos());
    let angle_pitch = angles[0] * DEG2RAD;
    let (sp, cp) = (angle_pitch.sin(), angle_pitch.cos());
    let angle_roll = angles[2] * DEG2RAD;
    let (sr, cr) = (angle_roll.sin(), angle_roll.cos());

    let forward = [cp * cy, cp * sy, -sp];
    let right = [
        -1.0 * sr * sp * cy + -1.0 * cr * -sy,
        -1.0 * sr * sp * sy + -1.0 * cr * cy,
        -1.0 * sr * cp,
    ];
    let up = [cr * sp * cy + -sr * -sy, cr * sp * sy + -sr * cy, cr * cp];
    (forward, right, up)
}

/// Raven `AnglesToAxis` (`q_math.c:530-536`): `axis[1] = vec3_origin - right`.
fn angles_to_axis(angles: vec3_t) -> [vec3_t; 3] {
    let (forward, right, up) = angle_vectors(angles);
    let axis1 = v_sub([0.0, 0.0, 0.0], right);
    [forward, axis1, up]
}

/// The identity `mdxaBone_t` (module-doc gap note #6's fallback for the
/// missing world matrix).
fn identity_mdxa_bone() -> mdxaBone_t {
    mdxaBone_t {
        matrix: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
        ],
    }
}

// ---------------------------------------------------------------------------
// list / name helpers
// ---------------------------------------------------------------------------

/// Raven `void G2_List_Model_Surfaces(const char *fileName)` — registers
/// `fileName`, then walks its `mdxm` surface-hierarchy table printing every
/// surface's name (+ descendants when `r_verbose` is set) via `Com_Printf`.
/// Debug lister; no failure path (a bad model would null-deref in the
/// oracle, matching the un-guarded `G2_List_Model_Bones` sibling below).
///
/// Divergence (§19, Raven UB site): a null `mod_m`/`mdxm` prints nothing here
/// instead of dereferencing null. Raven's dead advancing-`surface` local
/// (never read for content, only self-advanced) is dropped (§C10).
///
/// Source: `oracle/codemp/ghoul2/G2_misc.cpp:279-304`
pub fn g2_list_model_surfaces(host: &mut impl EngineHost, file_name: &str) {
    let model = register_model(host, file_name);
    let mdxm = host.model_mdxm(model);
    if mdxm.is_null() {
        return;
    }
    let num_surfaces = unsafe { read_i32(mdxm, MDXM_OFS_NUM_SURFACES) };
    let ofs_surf_hierarchy = unsafe { read_i32(mdxm, MDXM_OFS_OFS_SURF_HIERARCHY) };
    let verbose = host.cvar_integer("r_verbose") != 0;

    let mut surf = byte_add(mdxm, ofs_surf_hierarchy as usize);
    for x in 0..num_surfaces {
        let name = unsafe { read_cstr(surf, SURF_HIER_OFS_NAME, MAX_QPATH) };
        host.print(&format!("Surface {x} Name {name}\n"));
        let num_children = unsafe { read_i32(surf, SURF_HIER_OFS_NUM_CHILDREN) };
        if verbose {
            host.print(&format!("Num Descendants {num_children}\n"));
            for i in 0..num_children {
                let child = unsafe {
                    read_i32(
                        surf,
                        SURF_HIER_OFS_CHILD_INDEXES + (i as usize) * OFFSETS_ENTRY_SIZE,
                    )
                };
                host.print(&format!("Descendant {child}\n"));
            }
        }
        // find the next surface (Raven: `surf + &((mdxmSurfHierarchy_t*)0)->
        // childIndexes[surf->numChildren]`, i.e. this entry's fixed header
        // plus its variable `childIndexes` tail).
        surf = byte_add(
            surf,
            SURF_HIER_OFS_CHILD_INDEXES + (num_children as usize) * OFFSETS_ENTRY_SIZE,
        );
    }
}

/// Raven `void G2_List_Model_Bones(const char *fileName, int frame)` —
/// registers `fileName`, then walks its `mdxa` skeleton-offset table printing
/// every bone's name + base-pose position (+ descendant count when
/// `r_verbose` is set) via `Com_Printf`. `frame` is read into dead local
/// commented-out code (`:314-323`, the `mdxaFrame_t`/`frameSize` block) —
/// genuinely unused by the live body, kept as a parameter for 1:1 fidelity
/// (§A2).
///
/// Raven's own inner-loop body (`:335-339`) re-prints `skel->numChildren`
/// (not the child index) on every iteration — a copy/paste bug distinct from
/// `g2_list_model_surfaces`'s correct `Descendant %i` sibling — preserved
/// faithfully.
///
/// Source: `oracle/codemp/ghoul2/G2_misc.cpp:307-342`
pub fn g2_list_model_bones(host: &mut impl EngineHost, file_name: &str, frame: i32) {
    let _ = frame;
    let mod_m = register_model(host, file_name);
    let mdxm = host.model_mdxm(mod_m);
    if mdxm.is_null() {
        return;
    }
    let anim_index = unsafe { read_i32(mdxm, MDXM_OFS_ANIM_INDEX) };
    let header = host.model_mdxa(anim_index);
    if header.is_null() {
        return;
    }

    let num_bones = unsafe { read_i32(header, MDXA_OFS_NUM_BONES) };
    let verbose = host.cvar_integer("r_verbose") != 0;
    // Raven: `offsets = (mdxaSkelOffsets_t *)((byte *)header + sizeof(mdxaHeader_t));`
    let offsets = byte_add(header, MDXA_HEADER_SIZE);

    for x in 0..num_bones {
        let skel_offset = unsafe { read_i32(offsets, (x as usize) * OFFSETS_ENTRY_SIZE) };
        let skel = byte_add(offsets, skel_offset as usize);
        let name = unsafe { read_cstr(skel, MDXA_SKEL_OFS_NAME, MAX_QPATH) };
        host.print(&format!("Bone {x} Name {name}\n"));

        // `skel->BasePoseMat.matrix[0..2][3]` — the translation column.
        let px = unsafe { read_f32(skel, MDXA_SKEL_OFS_BASEPOSE + 12) };
        let py = unsafe { read_f32(skel, MDXA_SKEL_OFS_BASEPOSE + 28) };
        let pz = unsafe { read_f32(skel, MDXA_SKEL_OFS_BASEPOSE + 44) };
        host.print(&format!("X pos {px}, Y pos {py}, Z pos {pz}\n"));

        if verbose {
            let num_children = unsafe { read_i32(skel, MDXA_SKEL_OFS_NUM_CHILDREN) };
            host.print(&format!("Num Descendants {num_children}\n"));
            for _ in 0..num_children {
                // Raven copy/paste bug (module doc comment above): reprints
                // the parent's own child count, not the child's index.
                host.print(&format!("Num Descendants {num_children}\n"));
            }
        }
    }
}

/// Raven `qboolean G2_GetAnimFileName(const char *fileName, char **filename)`
/// — write-on-success-only (`G2SV-D1` generalized discriminator): `*filename`
/// is written only when the model, its `mdxm`, and a non-empty `animName` all
/// resolve, so the out-param collapses to `Option<String>` rather than a
/// write-through `&mut`.
///
/// Source: `oracle/codemp/ghoul2/G2_misc.cpp:356-367`
pub fn g2_get_anim_file_name(host: &mut impl EngineHost, file_name: &str) -> Option<String> {
    let model = register_model(host, file_name);
    let mdxm = host.model_mdxm(model);
    if mdxm.is_null() {
        return None;
    }
    let anim_name = unsafe { read_cstr(mdxm, MDXM_OFS_ANIM_NAME, MAX_QPATH) };
    if anim_name.is_empty() {
        return None;
    }
    Some(anim_name)
}

// ---------------------------------------------------------------------------
// model-pointer validity (G2_SetupModelPointers, two overloads)
// ---------------------------------------------------------------------------

/// Raven `qboolean G2_SetupModelPointers(CGhoul2Info *ghlInfo)` — the
/// single-instance overload every other roster file's wrappers open with:
/// re-derives `ghlInfo->currentModel`/`animModel`/`aHeader` via
/// `RE_RegisterModel`/`R_GetModelByHandle` (post-`vid_restart` revalidation),
/// setting `ghlInfo->mValid` to match. Named `g2_setup_model_pointers` (no
/// `_v` suffix) to disambiguate from the vector overload immediately below,
/// mirroring the crate's existing `_v`-suffix convention for the vector
/// siblings of singular Raven types (`CGhoul2Info_v`, `boneInfo_v`,
/// `surfaceInfo_v`).
///
/// **Doc/oracle citation gap (module-doc note #1):** actually *defined* at
/// `G2_API.cpp:2675-2693`, not `G2_misc.cpp:1839` (that line is only the
/// forward declaration); roster placement in `misc.rs` followed as written.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:2675-2693`
pub fn g2_setup_model_pointers(host: &mut impl EngineHost, ghl_info: &mut CGhoul2Info) -> bool {
    ghl_info.valid = false;
    if ghl_info.modelindex != -1 {
        let dedicated = host.cvar_integer("dedicated") != 0 || g2_should_register_server(host);
        ghl_info.model = if dedicated {
            register_server_model(host, &ghl_info.file_name)
        } else {
            register_model(host, &ghl_info.file_name)
        };

        let mdxm = host.model_mdxm(ghl_info.model);
        ghl_info.current_model = mdxm;
        if !mdxm.is_null() {
            // SAFETY: `mdxm` non-null, `EngineHost::model_mdxm`'s contract.
            let ofs_end = unsafe { read_i32(mdxm, MDXM_OFS_OFS_END) };
            if ghl_info.current_model_size != 0 && ghl_info.current_model_size != ofs_end {
                host.error(
                    errorParm_t::ERR_DROP,
                    "Ghoul2 model was reloaded and has changed, map must be restarted.\n",
                );
            }
            ghl_info.current_model_size = ofs_end;

            let anim_index = unsafe { read_i32(mdxm, MDXM_OFS_ANIM_INDEX) };
            let a_header = host.model_mdxa(anim_index);
            ghl_info.anim_model = a_header;
            if !a_header.is_null() {
                ghl_info.a_header = a_header;
                // SAFETY: `a_header` non-null, same contract as above.
                let a_ofs_end = unsafe { read_i32(a_header, MDXA_OFS_OFS_END) };
                if ghl_info.current_anim_model_size != 0
                    && ghl_info.current_anim_model_size != a_ofs_end
                {
                    host.error(
                        errorParm_t::ERR_DROP,
                        "Ghoul2 model was reloaded and has changed, map must be restarted.\n",
                    );
                }
                ghl_info.current_anim_model_size = a_ofs_end;
                ghl_info.valid = true;
            }
        }
    }
    if !ghl_info.valid {
        ghl_info.current_model = core::ptr::null();
        ghl_info.current_model_size = 0;
        ghl_info.anim_model = core::ptr::null();
        ghl_info.current_anim_model_size = 0;
        ghl_info.a_header = core::ptr::null();
    }
    ghl_info.valid
}

/// Raven `qboolean G2_SetupModelPointers(CGhoul2Info_v &ghoul2)` — the vector
/// overload: calls the single-instance overload on every arena-held instance,
/// `||`-accumulating the result (`qtrue` iff at least one instance is valid).
/// Needs `g2: &mut Ghoul2System` (not just `host`) to resolve the `ghoul2`
/// handle into its backing `Vec<CGhoul2Info>` (`Ghoul2InfoArray::get_mut`,
/// `info_array.rs`) before looping the single-instance overload over it.
///
/// **Doc/oracle citation gap (module-doc note #1):** actually *defined* at
/// `G2_API.cpp:2773-2783`, not `G2_misc.cpp:1839` (forward declaration only).
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:2773-2783`
pub fn g2_setup_model_pointers_v(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghoul2: &CGhoul2Info_v,
) -> bool {
    let mut ret = false;
    for i in 0..ghoul2.size(g2) {
        let r = g2_setup_model_pointers(host, ghoul2.get_mut(g2, i));
        ret = ret || r;
    }
    ret
}

// ---------------------------------------------------------------------------
// collision trace family (G2_TraceModels + private recursion helpers)
// ---------------------------------------------------------------------------

/// Raven `int G2_DecideTraceLod(CGhoul2Info &ghoul2, int useLod)` — clamps
/// `useLod` up to `ghoul2.mLodBias` and down to the model's highest valid LOD
/// (`currentModel->mdxm->numLODs - 1`). Private helper of `g2_trace_models`/
/// `g2_transform_model` below (no cross-file caller; `surfaces.rs`'s
/// `g2_add_surface` cites this same function by name but reads model memory
/// on its own `host`-threaded path, not by calling into this private fn).
///
/// Divergence (§19, Raven UB site): a null `mdxm` leaves `useLod` unclamped
/// instead of dereferencing null (the dropped `assert`s would have caught
/// this in a debug build; `NDEBUG` compiles them out).
///
/// Source: `oracle/codemp/ghoul2/G2_misc.cpp:376-398`
fn g2_decide_trace_lod(host: &mut impl EngineHost, ghoul2: &CGhoul2Info, use_lod: i32) -> i32 {
    let mut return_lod = use_lod;
    if ghoul2.lod_bias > return_lod {
        return_lod = ghoul2.lod_bias;
    }
    let mdxm = host.model_mdxm(ghoul2.model);
    if !mdxm.is_null() {
        // SAFETY: `mdxm` non-null, `EngineHost::model_mdxm`'s contract.
        let num_lods = unsafe { read_i32(mdxm, MDXM_OFS_NUM_LODS) };
        if return_lod >= num_lods {
            return_lod = num_lods - 1;
        }
    }
    return_lod
}

/// Raven `class CTraceSurface` (`G2_misc.cpp:190-275`) — the trace-recursion
/// scratch record `g2_trace_models` builds once per model instance and
/// threads through `g2_trace_surfaces`/`g2_trace_polys`/`g2_radius_trace_polys`.
/// Not ABI, not named on the doc's roster (a private helper of
/// `G2_TraceModels`, porting-rules §F17/CLAUDE.md "private helpers included");
/// container shape is free (§A1). Declared `pub` (not `pub(crate)`) only to
/// match the visibility of `gore::g2_gore_polys`, which takes it by `&mut`;
/// both are reached from the not-yet-wired collision `G2_TraceModels` loop. `current_model` is a `qhandle_t` (not a raw
/// pointer) matching the crate-wide convention of resolving model memory
/// through `EngineHost::model_mdxm`/`model_mdxa` on demand (`G2SV-D5`) rather
/// than caching an opaque pointer. The oracle's `skin`/`cust_shader` fields
/// back only the hit-location/hit-material lookup that is `/* ... */`-commented
/// out in `G2_TracePolys` (`:1151-1198`) — dead in this build, so dropped
/// here (§C10 dead-body-arm fold) rather than carried as unused fields.
///
/// `verts_cursor` (module-doc gap note #7, not a Raven field) is this port's
/// own bookkeeping: the running offset (in flat `f32` slots) into
/// `transformed_verts_array` the depth-first surface walk has consumed so
/// far, replacing Raven's `TS.TransformedVertsArray[thisSurfaceIndex]`
/// per-surface pointer lookup with a sequential cursor that lines up with the
/// identical-order walk `g2_transform_surfaces` used to fill the buffer.
pub struct CTraceSurface<'a> {
    pub(crate) surface_num: i32,
    pub(crate) root_slist: &'a [surfaceInfo_t],
    pub(crate) current_model: qhandle_t,
    pub(crate) lod: i32,
    pub(crate) ray_start: vec3_t,
    pub(crate) ray_end: vec3_t,
    pub(crate) coll_rec_map: &'a mut [CollisionRecord_t],
    pub(crate) ent_num: i32,
    pub(crate) model_index: i32,
    pub(crate) transformed_verts_array: *mut i32,
    pub(crate) trace_flags: i32,
    pub(crate) hit_one: bool,
    pub(crate) f_radius: f32,
    // gore-application fields — `_G2_GORE` on (`G2SV-D5`), always present.
    pub(crate) ssize: f32,
    pub(crate) tsize: f32,
    pub(crate) theta: f32,
    pub(crate) gore_shader: i32,
    pub(crate) ghoul2_info: *const CGhoul2Info,
    #[allow(dead_code)]
    pub(crate) gore: Option<&'a mut SSkinGoreData>,
    /// See the struct doc comment (module-doc gap note #7).
    pub(crate) verts_cursor: usize,
}

impl<'a> CTraceSurface<'a> {
    /// Raven `CTraceSurface::CTraceSurface(...)` member-init-list ctor —
    /// copies every `init*` argument into its like-named field, `VectorCopy`s
    /// `rayStart`/`rayEnd`, and seeds `hitOne = false`.
    ///
    /// Source: `oracle/codemp/ghoul2/G2_misc.cpp:221-273`
    #[allow(clippy::too_many_arguments)]
    fn new(
        surface_num: i32,
        root_slist: &'a [surfaceInfo_t],
        current_model: qhandle_t,
        lod: i32,
        ray_start: vec3_t,
        ray_end: vec3_t,
        coll_rec_map: &'a mut [CollisionRecord_t],
        ent_num: i32,
        model_index: i32,
        transformed_verts_array: *mut i32,
        trace_flags: i32,
        f_radius: f32,
        ssize: f32,
        tsize: f32,
        theta: f32,
        gore_shader: i32,
        ghoul2_info: *const CGhoul2Info,
        gore: Option<&'a mut SSkinGoreData>,
    ) -> Self {
        Self {
            surface_num,
            root_slist,
            current_model,
            lod,
            ray_start,
            ray_end,
            coll_rec_map,
            ent_num,
            model_index,
            transformed_verts_array,
            trace_flags,
            hit_one: false,
            f_radius,
            ssize,
            tsize,
            theta,
            gore_shader,
            ghoul2_info,
            gore,
            verts_cursor: 0,
        }
    }
}

/// Internal pointer-returning duplicate of the byte-walk [`g2_find_surface`]
/// (the public `i32`-returning overload) performs, for the two internal
/// callers that need the fuller record (`g2_trace_surfaces`/
/// `g2_transform_surfaces`) rather than just the resolved
/// `thisSurfaceIndex` (see that function's shape-choice doc comment).
///
/// Source: `oracle/codemp/ghoul2/G2_misc.cpp:1689-1713`
fn g2_find_surface_ptr(
    host: &mut impl EngineHost,
    model: qhandle_t,
    index: i32,
    lod: i32,
) -> *const c_void {
    let mdxm = host.model_mdxm(model);
    if mdxm.is_null() {
        return core::ptr::null();
    }
    let ofs_lods = unsafe { read_i32(mdxm, MDXM_OFS_OFS_LODS) };
    let mut current = byte_add(mdxm, ofs_lods as usize);
    for _ in 0..lod {
        let ofs_end = unsafe { read_i32(current, MDXM_LOD_OFS_END) };
        current = byte_add(current, ofs_end as usize);
    }
    current = byte_add(current, MDXM_LOD_SIZE);
    let offset = unsafe { read_i32(current, (index as usize) * OFFSETS_ENTRY_SIZE) };
    byte_add(current, offset as usize)
}

/// Raven `static void G2_TraceSurfaces(CTraceSurface &TS)` — resolves
/// `TS.surfaceNum`'s hierarchy entry (index-based [`g2_find_surface`]) and any
/// `TS.rootSList` override, then either radius- or point-traces the surface's
/// polys (`g2_radius_trace_polys`/`g2_trace_polys`); recurses into every child
/// surface unless `NODESCENDANTS` is set or `TS.hitOne` is already true.
///
/// **Fold (module-doc gap note #5, `G2SV-D7`):** the oracle's `if
/// (TS.collRecMap) {...trace...} else { G2_GorePolys(...) }` guard is dropped
/// to its permanent true arm — this port's `g2_trace_models` always supplies
/// a real `coll_rec_map` slice (never null), so the `G2_GorePolys` arm is
/// structurally unreachable and this function never calls into
/// `gore/gore_set.rs`.
///
/// `world_matrix` (module-doc gap note #6) is threaded explicitly since this
/// private helper's shape is free (§A1); `g2_trace_models` — whose frozen
/// signature has no way to receive the real matrix — passes the identity.
///
/// Source: `oracle/codemp/ghoul2/G2_misc.cpp:1428-1511`
fn g2_trace_surfaces(
    host: &mut impl EngineHost,
    ts: &mut CTraceSurface,
    world_matrix: &mdxaBone_t,
) {
    if ts.hit_one {
        return;
    }
    let mdxm = host.model_mdxm(ts.current_model);
    if mdxm.is_null() {
        return;
    }
    let surface = g2_find_surface_ptr(host, ts.current_model, ts.surface_num, ts.lod);
    if surface.is_null() {
        return;
    }
    let surf_indexes = byte_add(mdxm, MDXM_HEADER_SIZE);
    // SAFETY: `surface`/`surf_indexes` non-null, in-bounds per the loader's
    // model-memory contract (`EngineHost::model_mdxm`).
    let this_surface_index = unsafe { read_i32(surface, MDXM_SURF_OFS_THIS_SURFACE_INDEX) };
    let hier_offset = unsafe {
        read_i32(
            surf_indexes,
            (this_surface_index as usize) * OFFSETS_ENTRY_SIZE,
        )
    };
    let surf_info = byte_add(surf_indexes, hier_offset as usize);

    let surf_override = crate::surfaces::g2_find_override_surface(ts.surface_num, ts.root_slist);
    let mut off_flags = unsafe { read_u32(surf_info, SURF_HIER_OFS_FLAGS) } as i32;
    if let Some(over) = surf_override {
        off_flags = over.offFlags;
    }

    if off_flags == 0 {
        let num_verts = unsafe { read_i32(surface, MDXM_SURF_OFS_NUM_VERTS) };
        let base = ts.verts_cursor;
        ts.verts_cursor += (num_verts as usize) * 5;

        if !(ts.f_radius.abs() < 0.1) {
            if g2_radius_trace_polys(surface, base, ts, world_matrix)
                && ts.trace_flags == EG2_Collision::G2_RETURNONHIT as i32
            {
                ts.hit_one = true;
                return;
            }
        } else if g2_trace_polys(surface, base, ts, world_matrix)
            && ts.trace_flags == EG2_Collision::G2_RETURNONHIT as i32
        {
            ts.hit_one = true;
            return;
        }
    }

    if (off_flags as u32) & G2SURFACEFLAG_NODESCENDANTS != 0 {
        return;
    }

    let num_children = unsafe { read_i32(surf_info, SURF_HIER_OFS_NUM_CHILDREN) };
    for i in 0..num_children {
        if ts.hit_one {
            break;
        }
        let child = unsafe {
            read_i32(
                surf_info,
                SURF_HIER_OFS_CHILD_INDEXES + (i as usize) * OFFSETS_ENTRY_SIZE,
            )
        };
        ts.surface_num = child;
        g2_trace_surfaces(host, ts, world_matrix);
    }
}

/// Raven `static bool G2_TracePolys(const mdxmSurface_t *surface, const
/// mdxmSurfHierarchy_t *surfInfo, CTraceSurface &TS)` — point-trace: for each
/// triangle, [`g2_segment_triangle_test`]s the ray against it; on a hit,
/// claims a free `TS.collRecMap` slot, forcing `TS.hitOne = true` and
/// returning `true` if the map is full), fills in distance/flags/world-space
/// position+normal ([`transform_and_translate_point`]/[`transform_point`]) and
/// barycentric/UV via [`g2_build_hit_point_st`]. The commented-out
/// hit-location/hit-material shader lookup (`:1151-1198`) is dead (module
/// doc-comment note on `CTraceSurface`); `surfInfo` (the oracle's third
/// parameter) is genuinely unread by this function's own body, dropped from
/// this internal helper's signature per §A1 (its caller [`g2_trace_surfaces`]
/// already resolved it only to check `off_flags`, which this function never
/// needs). `host` is dropped too (no `EngineHost` service touched here,
/// same reasoning `gore/gore_set.rs`'s module doc applies to `G2_GorePolys`).
///
/// `base`/`world_matrix` are this port's own additions (module-doc gap notes
/// #6/#7); see [`g2_trace_surfaces`].
///
/// Source: `oracle/codemp/ghoul2/G2_misc.cpp:1092-1219`
fn g2_trace_polys(
    surface: *const c_void,
    base: usize,
    ts: &mut CTraceSurface,
    world_matrix: &mdxaBone_t,
) -> bool {
    let ofs_triangles = unsafe { read_i32(surface, MDXM_SURF_OFS_OFS_TRIANGLES) };
    let this_surface_index = unsafe { read_i32(surface, MDXM_SURF_OFS_THIS_SURFACE_INDEX) };
    let num_triangles = unsafe { read_i32(surface, MDXM_SURF_OFS_NUM_TRIANGLES) };
    let tris = byte_add(surface, ofs_triangles as usize);

    if ts.transformed_verts_array.is_null() {
        return false;
    }
    let verts_ptr = ts.transformed_verts_array as *const i32;

    for j in 0..num_triangles {
        let tri = byte_add(tris, (j as usize) * MDXM_TRIANGLE_SIZE);
        let i0 = unsafe { read_i32(tri, 0) } as usize;
        let i1 = unsafe { read_i32(tri, 4) } as usize;
        let i2 = unsafe { read_i32(tri, 8) } as usize;
        // SAFETY: `verts_ptr`/`base` are the flat buffer this model's
        // `g2_transform_surfaces` walk populated (module-doc gap note #7);
        // `i0`/`i1`/`i2` are in-bounds triangle vertex indices per the loader.
        let point1 = unsafe { read_flat_vert(verts_ptr, base, i0) };
        let point2 = unsafe { read_flat_vert(verts_ptr, base, i1) };
        let point3 = unsafe { read_flat_vert(verts_ptr, base, i2) };

        let Some((hit_point, normal, face)) = g2_segment_triangle_test(
            ts.ray_start,
            ts.ray_end,
            [point1[0], point1[1], point1[2]],
            [point2[0], point2[1], point2[2]],
            [point3[0], point3[1], point3[2]],
            true,
            true,
        ) else {
            continue;
        };

        let map_len = ts.coll_rec_map.len();
        let mut i = 0usize;
        while i < map_len && ts.coll_rec_map[i].mEntityNum != -1 {
            i += 1;
        }
        if i == map_len {
            ts.hit_one = true;
            return true;
        }

        let (s, t, bary_i, bary_j) = g2_build_hit_point_st(
            [point1[0], point1[1], point1[2]],
            point1[3],
            point1[4],
            [point2[0], point2[1], point2[2]],
            point2[3],
            point2[4],
            [point3[0], point3[1], point3[2]],
            point3[3],
            point3[4],
            hit_point,
        );
        let _ = (s, t);

        let dist_vect = v_sub(hit_point, ts.ray_start);
        let mut world_normal = transform_point(normal, world_matrix);
        v_normalize(&mut world_normal);

        let new_col = &mut ts.coll_rec_map[i];
        new_col.mPolyIndex = j;
        new_col.mEntityNum = ts.ent_num;
        new_col.mSurfaceIndex = this_surface_index;
        new_col.mModelIndex = ts.model_index;
        new_col.mFlags = if face > 0.0 {
            G2_FRONTFACE
        } else {
            G2_BACKFACE
        };
        new_col.mDistance = v_length(dist_vect);
        new_col.mCollisionPosition = transform_and_translate_point(hit_point, world_matrix);
        new_col.mCollisionNormal = world_normal;
        new_col.mMaterial = 0;
        new_col.mLocation = 0;
        new_col.mBarycentricI = bary_i;
        new_col.mBarycentricJ = bary_j;

        if ts.trace_flags == EG2_Collision::G2_RETURNONHIT as i32 {
            ts.hit_one = true;
            return true;
        }
    }
    false
}

/// Raven `static bool G2_RadiusTracePolys(const mdxmSurface_t *surface,
/// CTraceSurface &TS)` — the radius-trace twin of [`g2_trace_polys`]: sweeps a
/// cylinder of radius `TS.m_fRadius` along the ray against each triangle
/// instead of a bare segment test.
///
/// Source: `oracle/codemp/ghoul2/G2_misc.cpp:1222-1427`
fn g2_radius_trace_polys(
    surface: *const c_void,
    base: usize,
    ts: &mut CTraceSurface,
    world_matrix: &mdxaBone_t,
) -> bool {
    let mut basis2: vec3_t = [0.0, 0.0, 1.0];
    let v3_ray_dir_raw = v_sub(ts.ray_end, ts.ray_start);
    let mut basis1 = v_cross(v3_ray_dir_raw, basis2);

    if v_dot(basis1, basis1) < 0.1 {
        basis2 = [0.0, 1.0, 0.0];
        basis1 = v_cross(v3_ray_dir_raw, basis2);
    }
    let basis2 = v_cross(v3_ray_dir_raw, basis1);

    v_normalize(&mut basis1);
    let mut basis2 = basis2;
    v_normalize(&mut basis2);

    let c = 0.0f32.cos();
    let s = 0.0f32.sin();

    let taxis = v_add(
        v_scale(basis1, 0.5 * c / ts.f_radius),
        v_scale(basis2, 0.5 * s / ts.f_radius),
    );
    let saxis = v_add(
        v_scale(basis1, -0.5 * s / ts.f_radius),
        v_scale(basis2, 0.5 * c / ts.f_radius),
    );

    let num_verts = unsafe { read_i32(surface, MDXM_SURF_OFS_NUM_VERTS) };

    if ts.transformed_verts_array.is_null() {
        return false;
    }
    let verts_ptr = ts.transformed_verts_array as *const i32;

    let f = v_length_squared(v3_ray_dir_raw);
    let mut v3_ray_dir = v3_ray_dir_raw;
    if f != 0.0 {
        v3_ray_dir[0] /= f;
        v3_ray_dir[1] /= f;
        v3_ray_dir[2] /= f;
    }

    let mut flags: i32 = 63;
    let mut vert_flags = vec![0i32; num_verts.max(0) as usize];
    for j in 0..num_verts {
        // SAFETY: see `g2_trace_polys`'s identical-shape read.
        let vp = unsafe { read_flat_vert(verts_ptr, base, j as usize) };
        let delta = v_sub([vp[0], vp[1], vp[2]], ts.ray_start);
        let s_coord = v_dot(delta, saxis) + 0.5;
        let t_coord = v_dot(delta, taxis) + 0.5;
        let u_coord = v_dot(delta, v3_ray_dir);
        let mut vflags = 0i32;
        if s_coord > 0.0 {
            vflags |= 1;
        }
        if s_coord < 1.0 {
            vflags |= 2;
        }
        if t_coord > 0.0 {
            vflags |= 4;
        }
        if t_coord < 1.0 {
            vflags |= 8;
        }
        if u_coord > 0.0 {
            vflags |= 16;
        }
        if u_coord < 1.0 {
            vflags |= 32;
        }
        vflags = !vflags;
        flags &= vflags;
        vert_flags[j as usize] = vflags;
    }

    if flags != 0 {
        return false;
    }

    let num_triangles = unsafe { read_i32(surface, MDXM_SURF_OFS_NUM_TRIANGLES) };
    let ofs_triangles = unsafe { read_i32(surface, MDXM_SURF_OFS_OFS_TRIANGLES) };
    let tris = byte_add(surface, ofs_triangles as usize);
    let this_surface_index = unsafe { read_i32(surface, MDXM_SURF_OFS_THIS_SURFACE_INDEX) };

    for j in 0..num_triangles {
        let tri = byte_add(tris, (j as usize) * MDXM_TRIANGLE_SIZE);
        let i0 = unsafe { read_i32(tri, 0) } as usize;
        let i1 = unsafe { read_i32(tri, 4) } as usize;
        let i2 = unsafe { read_i32(tri, 8) } as usize;

        let tri_flags = 63 & vert_flags[i0] & vert_flags[i1] & vert_flags[i2];
        if tri_flags != 0 {
            continue;
        }

        let map_len = ts.coll_rec_map.len();
        let mut i = 0usize;
        while i < map_len && ts.coll_rec_map[i].mEntityNum != -1 {
            i += 1;
        }
        if i == map_len {
            ts.hit_one = true;
            return true;
        }

        let a = unsafe { read_flat_vert(verts_ptr, base, i0) };
        let b = unsafe { read_flat_vert(verts_ptr, base, i1) };
        let c_vert = unsafe { read_flat_vert(verts_ptr, base, i2) };
        let a3 = [a[0], a[1], a[2]];
        let b3 = [b[0], b[1], b[2]];
        let c3 = [c_vert[0], c_vert[1], c_vert[2]];

        let edge_ac = v_sub(c3, a3);
        let edge_ba = v_sub(b3, a3);
        let normal = v_cross(edge_ba, edge_ac);
        let mut world_normal = transform_point(normal, world_matrix);
        v_normalize(&mut world_normal);

        let dist_vect_dir = v_sub(ts.ray_end, ts.ray_start);
        let third = -(a3[0] * (b3[1] * c3[2] - c3[1] * b3[2])
            + b3[0] * (c3[1] * a3[2] - a3[1] * c3[2])
            + c3[0] * (a3[1] * b3[2] - b3[1] * a3[2]));
        let side = v_dot(normal, ts.ray_start) + third;
        let side2 = v_dot(normal, dist_vect_dir);
        let hit_point = if side2 != 0.0 {
            let dist = side / side2;
            let mut hp = ts.ray_start;
            hp[0] -= dist * dist_vect_dir[0];
            hp[1] -= dist * dist_vect_dir[1];
            hp[2] -= dist * dist_vect_dir[2];
            hp
        } else {
            ts.ray_start
        };

        let dist_vect = v_sub(hit_point, ts.ray_start);

        let new_col = &mut ts.coll_rec_map[i];
        new_col.mPolyIndex = j;
        new_col.mEntityNum = ts.ent_num;
        new_col.mSurfaceIndex = this_surface_index;
        new_col.mModelIndex = ts.model_index;
        new_col.mFlags = G2_FRONTFACE;
        new_col.mCollisionNormal = world_normal;
        new_col.mMaterial = 0;
        new_col.mLocation = 0;
        new_col.mDistance = v_length(dist_vect);
        new_col.mCollisionPosition = transform_and_translate_point(hit_point, world_matrix);
        new_col.mBarycentricI = 0.0;
        new_col.mBarycentricJ = 0.0;

        if ts.trace_flags == EG2_Collision::G2_RETURNONHIT as i32 {
            ts.hit_one = true;
            return true;
        }
    }

    false
}

/// Raven `static float G2_AreaOfTri(const vec3_t A, const vec3_t B, const
/// vec3_t C)` — twice the triangle's area (`|cross(A-B, C-B)|`), used as the
/// barycentric-coordinate denominator by [`g2_build_hit_point_st`]. Defined
/// unconditionally (not inside any `_G2_GORE` guard), so it is shared by both
/// [`g2_trace_polys`] here and `gore/gore_set.rs`'s `G2_GorePolys`.
///
/// Source: `oracle/codemp/ghoul2/G2_misc.cpp:665-674`
pub(crate) fn g2_area_of_tri(a: vec3_t, b: vec3_t, c: vec3_t) -> f32 {
    let ab = v_sub(a, b);
    let cb = v_sub(c, b);
    v_length(v_cross(ab, cb))
}

/// Raven `static void G2_BuildHitPointST(...)` — barycentric-interpolates the
/// triangle's UV coordinates at hit point `P` via [`g2_area_of_tri`], wrapping
/// `s`/`t` into `[0,1)`. All four out-params (`s`, `t`, `bary_i`, `bary_j`) are
/// written unconditionally on every path, so they collapse to a plain
/// returned tuple rather than write-through `&mut` refs (no failure path
/// exists to preserve).
///
/// Source: `oracle/codemp/ghoul2/G2_misc.cpp:677-705`
#[allow(clippy::too_many_arguments)]
fn g2_build_hit_point_st(
    a: vec3_t,
    sa: f32,
    ta: f32,
    b: vec3_t,
    sb: f32,
    tb: f32,
    c: vec3_t,
    sc: f32,
    tc: f32,
    p: vec3_t,
) -> (f32, f32, f32, f32) {
    let area_abc = g2_area_of_tri(a, b, c);

    let i = g2_area_of_tri(p, b, c) / area_abc;
    let bary_i = i;
    let j = g2_area_of_tri(a, p, c) / area_abc;
    let bary_j = j;
    let k = g2_area_of_tri(a, b, p) / area_abc;

    let mut s = sa * i + sb * j + sc * k;
    let mut t = ta * i + tb * j + tc * k;

    s %= 1.0;
    if s < 0.0 {
        s += 1.0;
    }
    t %= 1.0;
    if t < 0.0 {
        t += 1.0;
    }

    (s, t, bary_i, bary_j)
}

/// Raven `qboolean G2_SegmentTriangleTest(const vec3_t start, const vec3_t
/// end, const vec3_t A, const vec3_t B, const vec3_t C, qboolean backFaces,
/// qboolean frontFaces, vec3_t returnedPoint, vec3_t returnedNormal, float
/// *denom)` — ray/triangle intersection (plane test + three edge tests).
/// `returnedPoint`/`returnedNormal`/`denom` are written progressively as the
/// algorithm proceeds and are read by every in-crate caller (this file's
/// `g2_trace_polys`) only inside the `if (…)` success arm, never on a
/// `qfalse` return — a write-on-success-only shape (`G2SV-D1` generalized
/// discriminator) — so the three out-params collapse to one `Option<(vec3_t,
/// vec3_t, f32)>` = `Some((point, normal, denom))`. Non-`static` in the
/// oracle (external linkage) but has no cross-TU caller found in
/// `oracle/codemp/`; kept `pub(crate)` rather than `pub` since only this file
/// and `gore/gore_set.rs` (same crate) need it.
///
/// Source: `oracle/codemp/ghoul2/G2_misc.cpp:709-777`
#[allow(clippy::too_many_arguments)]
pub(crate) fn g2_segment_triangle_test(
    start: vec3_t,
    end: vec3_t,
    a: vec3_t,
    b: vec3_t,
    c: vec3_t,
    back_faces: bool,
    front_faces: bool,
) -> Option<(vec3_t, vec3_t, f32)> {
    const TINY: f32 = 1e-10;

    let edge_ac = v_sub(c, a);
    let normal_t = v_sub(b, a);
    let returned_normal = v_cross(normal_t, edge_ac);

    let ray = v_sub(end, start);
    let denom = v_dot(ray, returned_normal);

    if denom.abs() < TINY || (!back_faces && denom > 0.0) || (!front_faces && denom < 0.0) {
        return None;
    }

    let to_plane = v_sub(a, start);
    let t = v_dot(to_plane, returned_normal) / denom;
    if !(0.0..=1.0).contains(&t) {
        return None;
    }

    let scaled_ray = v_scale(ray, t);
    let returned_point = v_add(scaled_ray, start);

    let edge_pa = v_sub(a, returned_point);
    let edge_pb = v_sub(b, returned_point);
    let edge_pc = v_sub(c, returned_point);

    if v_dot(v_cross(edge_pa, edge_pb), returned_normal) < 0.0 {
        return None;
    }
    if v_dot(v_cross(edge_pc, edge_pa), returned_normal) < 0.0 {
        return None;
    }
    if v_dot(v_cross(edge_pb, edge_pc), returned_normal) < 0.0 {
        return None;
    }

    Some((returned_point, returned_normal, denom))
}

/// Raven `void G2_TraceModels(CGhoul2Info_v &ghoul2, vec3_t rayStart, vec3_t
/// rayEnd, CollisionRecord_t *collRecMap, int entNum, int eG2TraceType, int
/// useLod, float fRadius, float ssize, float tsize, float theta, int shader,
/// SSkinGoreData *gore, qboolean skipIfLODNotMatch)` — the `_G2_GORE`-on
/// overload (the only one that compiles, module doc-comment note). Walks
/// every valid, non-`GHOUL2_NOCOLLIDE` model instance, decides its LOD
/// ([`g2_decide_trace_lod`]), resets the surface-override lookup, builds a
/// [`CTraceSurface`], and recurses via [`g2_trace_surfaces`].
///
/// Module-doc gap note #6: uses the identity matrix in place of the real
/// `worldMatrix` (no signature path receives it). Module-doc gap note #5: the
/// second `if (!collRecMap && firstModelOnly) break;` (`G2_misc.cpp:1604`) is
/// dead here too — `coll_rec_map` is never null through this signature —
/// dropped (§C10). `shader`/`skin`/`cust_shader` locals feed only the dead
/// hit-material lookup (`CTraceSurface`'s own doc comment) and are dropped.
///
/// Source: `oracle/codemp/ghoul2/G2_misc.cpp:1514-1611`
#[allow(clippy::too_many_arguments)]
pub fn g2_trace_models(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghoul2: &mut CGhoul2Info_v,
    ray_start: vec3_t,
    ray_end: vec3_t,
    coll_rec_map: &mut [CollisionRecord_t],
    ent_num: i32,
    e_g2_trace_type: i32,
    use_lod: i32,
    f_radius: f32,
    ssize: f32,
    tsize: f32,
    theta: f32,
    shader: Option<qhandle_t>,
    mut gore: Option<&mut SSkinGoreData>,
    skip_if_lod_not_match: bool,
) {
    let world_matrix = identity_mdxa_bone();

    let count = ghoul2.size(g2);
    for i in 0..count {
        // Raven's `goreModelIndex=i;` trace-scoped scratch has no reader in
        // this port — its sole consumer, `G2_GorePolys`'s `GoreTagsTemp` key,
        // is never reached server-side (module-doc gap note #5) — dropped.
        if ghoul2.get(g2, i).modelindex == -1 {
            continue;
        }
        if !ghoul2.get(g2, i).valid {
            continue;
        }
        if ghoul2.get(g2, i).flags & GHOUL2_NOCOLLIDE != 0 {
            continue;
        }

        let lod = g2_decide_trace_lod(host, ghoul2.get(g2, i), use_lod);
        if skip_if_lod_not_match && lod != use_lod {
            continue;
        }

        let _ = crate::surfaces::g2_find_override_surface(-1, &ghoul2.get(g2, i).slist);

        let (surface_root, current_model) = {
            let info = ghoul2.get(g2, i);
            (info.surface_root, info.model)
        };
        let slist_ptr = ghoul2.get(g2, i).slist.as_ptr();
        let slist_len = ghoul2.get(g2, i).slist.len();
        // SAFETY: the walk below (`g2_trace_surfaces`) only reads the
        // override list; nothing mutates `ghoul2[i].slist` for the remainder
        // of this iteration (same pattern as `g2_transform_model`).
        let root_slist: &[surfaceInfo_t] =
            unsafe { core::slice::from_raw_parts(slist_ptr, slist_len) };
        let tva_ptr = ghoul2
            .get_mut(g2, i)
            .transformed_verts_array
            .as_mut()
            .map_or(core::ptr::null_mut(), |v| v.as_mut_ptr());
        let ghoul2_info_ptr = ghoul2.get(g2, i) as *const CGhoul2Info;

        let mut ts = CTraceSurface::new(
            surface_root,
            root_slist,
            current_model,
            lod,
            ray_start,
            ray_end,
            coll_rec_map,
            ent_num,
            i,
            tva_ptr,
            e_g2_trace_type,
            f_radius,
            ssize,
            tsize,
            theta,
            shader.unwrap_or(0),
            ghoul2_info_ptr,
            gore.as_mut().map(|r| &mut **r),
        );

        g2_trace_surfaces(host, &mut ts, &world_matrix);

        if ts.hit_one {
            break;
        }
    }
}

// ---------------------------------------------------------------------------
// transform family (G2_TransformModel + private recursion helpers)
// ---------------------------------------------------------------------------

/// Raven `void R_TransformEachSurface(const mdxmSurface_t *surface, vec3_t
/// scale, CMiniHeap *G2VertSpace, int *TransformedVertsArray, CBoneCache
/// *boneCache)` — deforms every vertex of one mesh surface by its lerped bone
/// weights (`EvalBoneCache`, the Seam-pinned free fn `eval_bone_cache` in
/// `render/bone_cache.rs`), appending the resulting `[x,y,z,s,t]` quintuples
/// to `transformed_verts` (module-doc gap note #7: this port's flat per-model
/// buffer, not Raven's per-surface `MiniHeapAlloc`'d block). `_XBOX` OFF
/// (`G2SV-D5` build config) — this is the only compiled arm.
///
/// `CMiniHeap *G2VertSpace` dropped (module-doc note #4); `host` is unused
/// (no `EngineHost` service touched in this range) but kept per the
/// already-declared signature.
///
/// Source: `oracle/codemp/ghoul2/G2_misc.cpp:404-513`
fn r_transform_each_surface(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    surface: *const c_void,
    scale: vec3_t,
    transformed_verts: &mut Vec<f32>,
    bone_cache: BoneCacheId,
) {
    let _ = host;

    let ofs_bone_references = unsafe { read_i32(surface, MDXM_SURF_OFS_OFS_BONE_REFS) };
    let bone_refs = byte_add(surface, ofs_bone_references as usize);
    let num_verts = unsafe { read_i32(surface, MDXM_SURF_OFS_NUM_VERTS) };
    let ofs_verts = unsafe { read_i32(surface, MDXM_SURF_OFS_OFS_VERTS) };
    let verts_base = byte_add(surface, ofs_verts as usize);
    let texcoords_base = byte_add(verts_base, (num_verts as usize) * MDXM_VERTEX_SIZE);

    let scale_needed = scale[0] != 1.0 || scale[1] != 1.0 || scale[2] != 1.0;

    for j in 0..num_verts {
        let vert = byte_add(verts_base, (j as usize) * MDXM_VERTEX_SIZE);
        // SAFETY: `vert` is `j < numVerts` into the loader's parsed block.
        let normal: vec3_t = unsafe { [read_f32(vert, 0), read_f32(vert, 4), read_f32(vert, 8)] };
        let vert_coords: vec3_t =
            unsafe { [read_f32(vert, 12), read_f32(vert, 16), read_f32(vert, 20)] };
        let packed = unsafe { read_u32(vert, 24) };
        let bone_weightings: [u8; 4] = unsafe {
            let p = (vert as *const u8).add(28);
            [*p, *p.add(1), *p.add(2), *p.add(3)]
        };

        // Raven `G2_GetVertWeights`/`G2_GetVertBoneIndex`/`G2_GetVertBoneWeight`
        // (`mdx_format.h:266-297`).
        let num_weights = (packed >> 30) + 1;
        let mut temp_vert = [0.0f32; 3];
        let mut temp_normal = [0.0f32; 3];
        let mut total_weight = 0.0f32;

        for k in 0..num_weights {
            let bone_index = ((packed >> (5 * k)) & 0x1F) as i32;
            let bone_weight = if k == num_weights - 1 {
                1.0 - total_weight
            } else {
                let mut w = bone_weightings[k as usize] as u32;
                w |= (packed >> (12 + k * 2)) & 0x300;
                let weight = (1.0f32 / 1023.0f32) * (w as f32);
                total_weight += weight;
                weight
            };

            let bone_ref = unsafe { read_i32(bone_refs, (bone_index as usize) * 4) };
            let bone = eval_bone_cache(g2, bone_cache, bone_ref);

            for r in 0..3 {
                temp_vert[r] +=
                    bone_weight * (v_dot(row3(bone.matrix[r]), vert_coords) + bone.matrix[r][3]);
                temp_normal[r] += bone_weight * v_dot(row3(bone.matrix[r]), normal);
            }
        }
        let _ = temp_normal;

        let tex: [f32; 2] = unsafe {
            [
                read_f32(texcoords_base, (j as usize) * 8),
                read_f32(texcoords_base, (j as usize) * 8 + 4),
            ]
        };

        if scale_needed {
            transformed_verts.push(temp_vert[0] * scale[0]);
            transformed_verts.push(temp_vert[1] * scale[1]);
            transformed_verts.push(temp_vert[2] * scale[2]);
        } else {
            transformed_verts.push(temp_vert[0]);
            transformed_verts.push(temp_vert[1]);
            transformed_verts.push(temp_vert[2]);
        }
        transformed_verts.push(tex[0]);
        transformed_verts.push(tex[1]);
    }
}

/// Raven `void G2_TransformSurfaces(int surfaceNum, surfaceInfo_v &rootSList,
/// CBoneCache *boneCache, const model_t *currentModel, int lod, vec3_t scale,
/// CMiniHeap *G2VertSpace, int *TransformedVertArray, bool
/// secondTimeAround)` — resolves `surfaceNum`'s hierarchy entry (index-based
/// [`g2_find_surface_ptr`]), applies any `rootSList` override
/// (`crate::surfaces::g2_find_override_surface`), forwards to
/// [`r_transform_each_surface`] when the surface isn't flagged off, then
/// recurses into every child surface unless `NODESCENDANTS` is set.
/// `secondTimeAround` is passed `false` at its only call site
/// (`G2_TransformModel`, `:651`) — never actually toggled — but kept as a
/// parameter for 1:1 fidelity (§A2: don't drop a real, if unread, parameter).
///
/// `CMiniHeap`/`model_t` dropped/opaqued per this file's module-doc notes.
///
/// Source: `oracle/codemp/ghoul2/G2_misc.cpp:517-556`
#[allow(clippy::too_many_arguments)]
fn g2_transform_surfaces(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    surface_num: i32,
    root_slist: &[surfaceInfo_t],
    bone_cache: BoneCacheId,
    current_model: qhandle_t,
    lod: i32,
    scale: vec3_t,
    transformed_verts: &mut Vec<f32>,
    second_time_around: bool,
) {
    let _ = second_time_around;

    let surface = g2_find_surface_ptr(host, current_model, surface_num, lod);
    if surface.is_null() {
        return;
    }
    let mdxm = host.model_mdxm(current_model);
    if mdxm.is_null() {
        return;
    }
    let surf_indexes = byte_add(mdxm, MDXM_HEADER_SIZE);
    let this_surface_index = unsafe { read_i32(surface, MDXM_SURF_OFS_THIS_SURFACE_INDEX) };
    let hier_offset = unsafe {
        read_i32(
            surf_indexes,
            (this_surface_index as usize) * OFFSETS_ENTRY_SIZE,
        )
    };
    let surf_info = byte_add(surf_indexes, hier_offset as usize);

    let surf_override = crate::surfaces::g2_find_override_surface(surface_num, root_slist);
    let mut off_flags = unsafe { read_u32(surf_info, SURF_HIER_OFS_FLAGS) } as i32;
    if let Some(over) = surf_override {
        off_flags = over.offFlags;
    }

    if off_flags == 0 {
        r_transform_each_surface(g2, host, surface, scale, transformed_verts, bone_cache);
    }

    if (off_flags as u32) & G2SURFACEFLAG_NODESCENDANTS != 0 {
        return;
    }

    let num_children = unsafe { read_i32(surf_info, SURF_HIER_OFS_NUM_CHILDREN) };
    for i in 0..num_children {
        let child = unsafe {
            read_i32(
                surf_info,
                SURF_HIER_OFS_CHILD_INDEXES + (i as usize) * OFFSETS_ENTRY_SIZE,
            )
        };
        g2_transform_surfaces(
            g2,
            host,
            child,
            root_slist,
            bone_cache,
            current_model,
            lod,
            scale,
            transformed_verts,
            second_time_around,
        );
    }
}

/// Raven `void G2_TransformModel(CGhoul2Info_v &ghoul2, const int frameNum,
/// vec3_t scale, CMiniHeap *G2VertSpace, int useLod, bool ApplyGore)` — the
/// `_G2_GORE`-on overload (the only one that compiles). Server-live via
/// `G2API_CollisionDetect`/`CollisionDetectCache` (`api_collision.rs`,
/// `apply_gore = false`) and the graph-dead `G2API_AddSkinGore`
/// (`apply_gore = true`, `G2SV-D7`). Walks every valid instance, corrects a
/// zero `scale` axis up to `1.0`, decides the LOD (`useLod` directly when
/// `ApplyGore`, else [`g2_decide_trace_lod`]), builds the flat transformed-verts
/// buffer (module-doc gap note #7), resets the surface-override lookup, and
/// recurses via [`g2_transform_surfaces`].
///
/// `CMiniHeap` dropped (module-doc note #4). The `GHOUL2_ZONETRANSALLOC`-gated
/// reuse-vs-rebuild optimization (`G2_misc.cpp:637-646`) collapses to "always
/// rebuild" here — this port's flat buffer has no partial-reuse concept, a
/// performance-only divergence (the observable per-frame result is identical).
///
/// Source: `oracle/codemp/ghoul2/G2_misc.cpp:558-661`
#[allow(clippy::too_many_arguments)]
pub fn g2_transform_model(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghoul2: &mut CGhoul2Info_v,
    frame_num: i32,
    scale: vec3_t,
    use_lod: i32,
    apply_gore: bool,
) {
    let first_model_only = host.cvar_integer("cg_g2MarksAllModels") == 0;

    let mut correct_scale = scale;
    if correct_scale[0] == 0.0 {
        correct_scale[0] = 1.0;
    }
    if correct_scale[1] == 0.0 {
        correct_scale[1] = 1.0;
    }
    if correct_scale[2] == 0.0 {
        correct_scale[2] = 1.0;
    }

    let count = ghoul2.size(g2);
    for i in 0..count {
        if !ghoul2.get(g2, i).valid {
            continue;
        }
        ghoul2.get_mut(g2, i).mesh_frame_num = frame_num;

        let model = ghoul2.get(g2, i).model;

        let lod = if apply_gore {
            let mdxm = host.model_mdxm(model);
            let num_lods = if mdxm.is_null() {
                0
            } else {
                // Divergence: reads `mdxmHeader_t::numLODs` in place of the
                // separate `model_t::numLods` cache Raven checks here
                // (`G2_misc.cpp:616`) — `EngineHost` exposes only the parsed
                // `.glm` block, not `model_t` itself (`G2SV-D5`); the two
                // agree for any model the loader actually built.
                unsafe { read_i32(mdxm, MDXM_OFS_NUM_LODS) }
            };
            if use_lod >= num_lods {
                ghoul2.get_mut(g2, i).transformed_verts_array = None;
                if first_model_only {
                    return;
                }
                continue;
            }
            use_lod
        } else {
            let info_ptr = ghoul2.get(g2, i) as *const CGhoul2Info;
            // SAFETY: `info_ptr` is the same arena slot the surrounding loop
            // already borrows immutably; no mutation happens before use.
            g2_decide_trace_lod(host, unsafe { &*info_ptr }, use_lod)
        };

        let _ = crate::surfaces::g2_find_override_surface(-1, &ghoul2.get(g2, i).slist);

        let (surface_root, current_model) = {
            let info = ghoul2.get(g2, i);
            (info.surface_root, info.model)
        };
        let slist_ptr = ghoul2.get(g2, i).slist.as_ptr();
        let slist_len = ghoul2.get(g2, i).slist.len();
        // SAFETY: the recursive walk only reads overrides; `ghoul2[i].slist`
        // is not mutated for the remainder of this iteration.
        let root_slist: &[surfaceInfo_t] =
            unsafe { core::slice::from_raw_parts(slist_ptr, slist_len) };
        let bone_cache = ghoul2.get(g2, i).bone_cache;

        let mut transformed_verts: Vec<f32> = Vec::new();
        if let Some(cache) = bone_cache {
            g2_transform_surfaces(
                g2,
                host,
                surface_root,
                root_slist,
                cache,
                current_model,
                lod,
                correct_scale,
                &mut transformed_verts,
                false,
            );
        }

        let bits: Vec<i32> = transformed_verts
            .iter()
            .map(|f| f.to_bits() as i32)
            .collect();
        ghoul2.get_mut(g2, i).transformed_verts_array = Some(bits);

        if apply_gore && first_model_only {
            break;
        }
    }
}

// ---------------------------------------------------------------------------
// world / inverse matrix math
// ---------------------------------------------------------------------------

/// Raven `void Create_Matrix(const float *angle, mdxaBone_t *matrix)` —
/// `AnglesToAxis` + pack into a rotation-only `mdxaBone_t` (translation column
/// zeroed). Private helper of [`g2_generate_world_matrix`]; no cross-file
/// caller.
///
/// Source: `oracle/codemp/ghoul2/G2_misc.cpp:1630-1653`
fn create_matrix(angle: vec3_t) -> mdxaBone_t {
    let axis = angles_to_axis(angle);
    let mut matrix = [[0.0f32; 4]; 3];
    matrix[0][0] = axis[0][0];
    matrix[1][0] = axis[0][1];
    matrix[2][0] = axis[0][2];

    matrix[0][1] = axis[1][0];
    matrix[1][1] = axis[1][1];
    matrix[2][1] = axis[1][2];

    matrix[0][2] = axis[2][0];
    matrix[1][2] = axis[2][1];
    matrix[2][2] = axis[2][2];

    matrix[0][3] = 0.0;
    matrix[1][3] = 0.0;
    matrix[2][3] = 0.0;

    mdxaBone_t { matrix }
}

/// Raven `void G2_GenerateWorldMatrix(const vec3_t angles, const vec3_t
/// origin)` — builds the per-construct `worldMatrix`/`worldMatrixInv` pair
/// (`tr_ghoul2.cpp:136-137`) via [`create_matrix`] + [`inverse_matrix`].
/// Per `## State ownership`, `worldMatrix`/`worldMatrixInv` are **not** a
/// `Ghoul2System` field — they are per-construct scratch threaded through the
/// skeleton build — so this returns the `(world_matrix, world_matrix_inv)`
/// pair rather than writing into subsystem state; the caller
/// (`G2_ConstructGhoulSkeleton`, `render/skeleton.rs`) threads the pair into
/// the transform chain.
///
/// Source: `oracle/codemp/ghoul2/G2_misc.cpp:1678-1686`
pub fn g2_generate_world_matrix(angles: vec3_t, origin: vec3_t) -> (mdxaBone_t, mdxaBone_t) {
    let mut world_matrix = create_matrix(angles);
    world_matrix.matrix[0][3] = origin[0];
    world_matrix.matrix[1][3] = origin[1];
    world_matrix.matrix[2][3] = origin[2];

    let world_matrix_inv = inverse_matrix(&world_matrix);
    (world_matrix, world_matrix_inv)
}

/// Raven `void TransformPoint(const vec3_t in, vec3_t out, mdxaBone_t *mat)`
/// — rotate `in` by `mat` (no translation). Bare global name in the oracle
/// (no `G2_` prefix); the out-param becomes a return per §C7.
///
/// Source: `oracle/codemp/ghoul2/G2_misc.cpp:1613-1618`
pub fn transform_point(input: vec3_t, mat: &mdxaBone_t) -> vec3_t {
    let mut out = [0.0f32; 3];
    for i in 0..3 {
        out[i] =
            input[0] * mat.matrix[i][0] + input[1] * mat.matrix[i][1] + input[2] * mat.matrix[i][2];
    }
    out
}

/// Raven `void TransformAndTranslatePoint (const vec3_t in, vec3_t out,
/// mdxaBone_t *mat)` — rotate **and** translate `in` by `mat`. Bare global
/// name in the oracle; the out-param becomes a return per §C7.
///
/// Source: `oracle/codemp/ghoul2/G2_misc.cpp:1620-1626`
pub fn transform_and_translate_point(input: vec3_t, mat: &mdxaBone_t) -> vec3_t {
    let mut out = [0.0f32; 3];
    for i in 0..3 {
        out[i] = input[0] * mat.matrix[i][0]
            + input[1] * mat.matrix[i][1]
            + input[2] * mat.matrix[i][2]
            + mat.matrix[i][3];
    }
    out
}

/// Raven `void Inverse_Matrix(mdxaBone_t *src, mdxaBone_t *dest)` —
/// transpose the 3x3 rotation block, then solve the translation column so
/// `dest` is `src`'s inverse (rigid transform: no scale/shear). Bare global
/// name in the oracle; the out-param becomes a return per §C7.
///
/// Source: `oracle/codemp/ghoul2/G2_misc.cpp:1656-1675`
pub fn inverse_matrix(src: &mdxaBone_t) -> mdxaBone_t {
    let mut dest = mdxaBone_t {
        matrix: [[0.0f32; 4]; 3],
    };
    for i in 0..3 {
        for j in 0..3 {
            dest.matrix[i][j] = src.matrix[j][i];
        }
    }
    for i in 0..3 {
        dest.matrix[i][3] = 0.0;
        for j in 0..3 {
            dest.matrix[i][3] -= dest.matrix[i][j] * src.matrix[j][3];
        }
    }
    dest
}

// ---------------------------------------------------------------------------
// index-based surface locator
// ---------------------------------------------------------------------------

/// Raven `void *G2_FindSurface(void *mod_t, int index, int lod)` — walks
/// `mod->mdxm`'s per-LOD surface-offset table down to LOD `lod`, then indexes
/// `index` within it, returning a pointer into the surface's `mdxmSurface_t`
/// record. The **index-based** overload — distinct from `G2_surfaces.cpp`'s
/// private *name-based* `G2_FindSurface(CGhoul2Info*, surfaceInfo_v&, const
/// char*, int*)`, which `surfaces.rs` owns as `g2_find_surface_by_name`
/// (module-doc note #3).
///
/// **Shape choice (§A1, not doc-pinned).** Every confirmed cross-file caller
/// reads only the resolved `surf->thisSurfaceIndex` off the returned pointer,
/// never any other field or the pointer identity — this returns that one
/// `i32` rather than an opaque pointer this crate could never usefully
/// dereference (`mdxmSurface_t` is never nameable, `G2SV-D5`).
///
/// Divergence (§19, Raven UB site): a null `mdxm` returns `0` here instead of
/// dereferencing null (the oracle has no found/not-found signal at all).
///
/// Source: `oracle/codemp/ghoul2/G2_misc.cpp:1689-1713`
pub fn g2_find_surface(host: &mut impl EngineHost, model: qhandle_t, index: i32, lod: i32) -> i32 {
    let surf = g2_find_surface_ptr(host, model, index, lod);
    if surf.is_null() {
        return 0;
    }
    unsafe { read_i32(surf, MDXM_SURF_OFS_THIS_SURFACE_INDEX) }
}

// ---------------------------------------------------------------------------
// save / load (de)serialization
// ---------------------------------------------------------------------------

/// This port's fixed field order for the "save-serialized middle band"
/// (`CGhoul2Info.modelindex..flags`, `## State ownership`/`shared/
/// cghoul2_info.rs`'s own doc comment). **Not** byte-compatible with Raven's
/// raw `memcpy` (which relies on `#[repr(C)]` layout this idiomatic
/// `CGhoul2Info` — porting-rules §F17 — does not have, and a fixed-size
/// `char[MAX_QPATH]` this port's owned `String` field does not have either);
/// a self-consistent Rust-native round-trip is what [`g2_save_ghoul2_models`]/
/// [`g2_load_ghoul2_model`] need, since `api_saveload.rs`'s own module doc
/// comment (gap note #2) already finds six of this file's seven roster
/// callers graph-dead server-side (no `G_G2_*` trap reaches them) — there is
/// no oracle-vs-Rust golden this pair needs to match byte-for-byte.
fn write_cghoul2_info_middle_band(buf: &mut Vec<u8>, info: &CGhoul2Info) {
    buf.extend(info.modelindex.to_ne_bytes());
    buf.extend(info.custom_shader.to_ne_bytes());
    buf.extend(info.custom_skin.to_ne_bytes());
    buf.extend(info.model_bolt_link.to_ne_bytes());
    buf.extend(info.surface_root.to_ne_bytes());
    buf.extend(info.lod_bias.to_ne_bytes());
    buf.extend(info.new_origin.to_ne_bytes());
    buf.extend(info.gore_set_tag.to_ne_bytes());
    buf.extend(info.model.to_ne_bytes());
    let mut name_bytes = [0u8; MAX_QPATH];
    let src = info.file_name.as_bytes();
    let n = src.len().min(MAX_QPATH - 1);
    name_bytes[..n].copy_from_slice(&src[..n]);
    buf.extend(name_bytes);
    buf.extend(info.anim_frame_default.to_ne_bytes());
    buf.extend(info.skel_frame_num.to_ne_bytes());
    buf.extend(info.mesh_frame_num.to_ne_bytes());
    buf.extend(info.flags.to_ne_bytes());
}

fn read_i32_seq(buf: &[u8], pos: &mut usize) -> i32 {
    let v = i32::from_ne_bytes(buf[*pos..*pos + 4].try_into().unwrap());
    *pos += 4;
    v
}

/// Raven `qboolean G2_SaveGhoul2Models(CGhoul2Info_v &ghoul2, char **buffer,
/// int *size)` — flattens every instance's save-serialized middle band
/// plus its surface/bone/bolt lists into one buffer (a 4-byte zero-count
/// buffer when `ghoul2` is empty). Called by `api_saveload.rs`'s
/// `g2api_save_ghoul2_models`, whose module-doc note already covers the
/// `G2SV-D1` discriminator mismatch (no real failure path — this always
/// returns `Some`); this internal fn mirrors that shape 1:1. Pure in-memory
/// serialization, no `host` touch in the oracle.
///
/// Not byte-compatible with the oracle's raw `memcpy` layout — see
/// [`write_cghoul2_info_middle_band`]'s doc comment. `surfaceInfo_t`/
/// `boneInfo_t`/`boltInfo_t` (the three sub-lists) ARE `#[repr(C)]`-frozen, so
/// those three are still serialized by raw byte copy, matching Raven exactly.
///
/// Source: `oracle/codemp/ghoul2/G2_misc.cpp:1719-1809`
pub fn g2_save_ghoul2_models(g2: &Ghoul2System, ghoul2: &CGhoul2Info_v) -> Option<Vec<u8>> {
    let count = ghoul2.size(g2);
    if count == 0 {
        return Some(vec![0u8; 4]);
    }

    let mut buf = Vec::new();
    buf.extend(count.to_ne_bytes());

    for i in 0..count {
        let info = ghoul2.get(g2, i);
        write_cghoul2_info_middle_band(&mut buf, info);

        buf.extend((info.slist.len() as i32).to_ne_bytes());
        for s in &info.slist {
            // SAFETY: `surfaceInfo_t` is `#[repr(C)]` with no padding bytes
            // that would leak uninitialized memory (`shared/surface_info_t.rs`).
            buf.extend(unsafe {
                core::slice::from_raw_parts(
                    (s as *const surfaceInfo_t) as *const u8,
                    core::mem::size_of::<surfaceInfo_t>(),
                )
            });
        }

        buf.extend((info.blist.len() as i32).to_ne_bytes());
        for b in &info.blist {
            // SAFETY: `boneInfo_t` is `#[repr(C)]` (`shared/bone_info_t.rs`).
            buf.extend(unsafe {
                core::slice::from_raw_parts(
                    (b as *const boneInfo_t) as *const u8,
                    core::mem::size_of::<boneInfo_t>(),
                )
            });
        }

        buf.extend((info.bltlist.len() as i32).to_ne_bytes());
        for bo in &info.bltlist {
            // SAFETY: `boltInfo_t` is `#[repr(C)]` (`shared/bolt_info_t.rs`).
            buf.extend(unsafe {
                core::slice::from_raw_parts(
                    (bo as *const boltInfo_t) as *const u8,
                    core::mem::size_of::<boltInfo_t>(),
                )
            });
        }
    }

    Some(buf)
}

/// Raven `void G2_LoadGhoul2Model(CGhoul2Info_v &ghoul2, char *buffer)` —
/// resizes `ghoul2` to the leading instance count (no-op early return when
/// that count is `0`), then walks `buffer` rebuilding each instance's model
/// index/filename/surface/bone/bolt lists, re-deriving model pointers via
/// [`g2_setup_model_pointers`] on any instance whose `mModelindex`/`mFileName`
/// resolved. Matches `api_saveload.rs`'s `g2api_load_ghoul2_models` 1:1
/// (`host` is threaded solely for the `g2_setup_model_pointers`
/// re-derivation this body performs).
///
/// Reads the fixed field order [`write_cghoul2_info_middle_band`] writes —
/// see that function's doc comment on why this is not oracle-byte-compatible.
///
/// Source: `oracle/codemp/ghoul2/G2_misc.cpp:1841-1910`
pub fn g2_load_ghoul2_model(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghoul2: &mut CGhoul2Info_v,
    buffer: &[u8],
) {
    let mut pos = 0usize;
    let new_size = read_i32_seq(buffer, &mut pos);
    ghoul2.resize(g2, new_size);

    if new_size == 0 {
        return;
    }

    for i in 0..ghoul2.size(g2) {
        {
            let info = ghoul2.get_mut(g2, i);
            info.skel_frame_num = 0;
            info.modelindex = -1;
            info.file_name.clear();
            info.valid = false;
        }

        let modelindex = read_i32_seq(buffer, &mut pos);
        let custom_shader = read_i32_seq(buffer, &mut pos);
        let custom_skin = read_i32_seq(buffer, &mut pos);
        let model_bolt_link = read_i32_seq(buffer, &mut pos);
        let surface_root = read_i32_seq(buffer, &mut pos);
        let lod_bias = read_i32_seq(buffer, &mut pos);
        let new_origin = read_i32_seq(buffer, &mut pos);
        let gore_set_tag = read_i32_seq(buffer, &mut pos);
        let model = read_i32_seq(buffer, &mut pos);
        let name_bytes = &buffer[pos..pos + MAX_QPATH];
        pos += MAX_QPATH;
        let end = name_bytes.iter().position(|&b| b == 0).unwrap_or(MAX_QPATH);
        let file_name = String::from_utf8_lossy(&name_bytes[..end]).into_owned();
        let anim_frame_default = read_i32_seq(buffer, &mut pos);
        let skel_frame_num = read_i32_seq(buffer, &mut pos);
        let mesh_frame_num = read_i32_seq(buffer, &mut pos);
        let flags = read_i32_seq(buffer, &mut pos);

        {
            let info = ghoul2.get_mut(g2, i);
            info.modelindex = modelindex;
            info.custom_shader = custom_shader;
            info.custom_skin = custom_skin;
            info.model_bolt_link = model_bolt_link;
            info.surface_root = surface_root;
            info.lod_bias = lod_bias;
            info.new_origin = new_origin;
            info.gore_set_tag = gore_set_tag;
            info.model = model;
            info.file_name = file_name;
            info.anim_frame_default = anim_frame_default;
            info.skel_frame_num = skel_frame_num;
            info.mesh_frame_num = mesh_frame_num;
            info.flags = flags;
        }

        if ghoul2.get(g2, i).modelindex != -1 && !ghoul2.get(g2, i).file_name.is_empty() {
            ghoul2.get_mut(g2, i).modelindex = i;
            g2_setup_model_pointers(host, ghoul2.get_mut(g2, i));
        }

        let num_surfaces = read_i32_seq(buffer, &mut pos);
        {
            let info = ghoul2.get_mut(g2, i);
            info.slist.clear();
            for _ in 0..num_surfaces {
                // SAFETY: `surfaceInfo_t` is `#[repr(C)]`, POD, no `Drop`; the
                // buffer holds exactly what `g2_save_ghoul2_models` wrote.
                let s: surfaceInfo_t = unsafe {
                    core::ptr::read_unaligned(buffer[pos..].as_ptr() as *const surfaceInfo_t)
                };
                pos += core::mem::size_of::<surfaceInfo_t>();
                info.slist.push(s);
            }
        }

        let num_bones = read_i32_seq(buffer, &mut pos);
        {
            let info = ghoul2.get_mut(g2, i);
            info.blist.clear();
            for _ in 0..num_bones {
                // SAFETY: see the `surfaceInfo_t` read above; `boneInfo_t` is
                // likewise `#[repr(C)]` POD.
                let b: boneInfo_t = unsafe {
                    core::ptr::read_unaligned(buffer[pos..].as_ptr() as *const boneInfo_t)
                };
                pos += core::mem::size_of::<boneInfo_t>();
                info.blist.push(b);
            }
        }

        let num_bolts = read_i32_seq(buffer, &mut pos);
        {
            let info = ghoul2.get_mut(g2, i);
            info.bltlist.clear();
            for _ in 0..num_bolts {
                // SAFETY: see the `surfaceInfo_t` read above; `boltInfo_t` is
                // likewise `#[repr(C)]` POD.
                let bo: boltInfo_t = unsafe {
                    core::ptr::read_unaligned(buffer[pos..].as_ptr() as *const boltInfo_t)
                };
                pos += core::mem::size_of::<boltInfo_t>();
                info.bltlist.push(bo);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mp_host_interface::mock::MockHost;

    // --- vector math ---------------------------------------------------------

    #[test]
    fn dot_and_cross_match_standard_formulas() {
        assert_eq!(v_dot([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]), 32.0);
        assert_eq!(v_cross([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]), [0.0, 0.0, 1.0]);
    }

    #[test]
    fn normalize_scales_to_unit_length_and_reports_original_length() {
        let mut v = [3.0f32, 4.0, 0.0];
        let len = v_normalize(&mut v);
        assert_eq!(len, 5.0);
        assert!((v_length(v) - 1.0).abs() < 1e-6);
    }

    // --- matrix math (Create_Matrix/Inverse_Matrix/G2_GenerateWorldMatrix) ---
    // Source: `oracle/codemp/ghoul2/G2_misc.cpp:1630-1686`

    #[test]
    fn create_matrix_at_zero_angles_is_identity_rotation() {
        let m = create_matrix([0.0, 0.0, 0.0]);
        let id = identity_mdxa_bone();
        for r in 0..3 {
            for c in 0..4 {
                assert!((m.matrix[r][c] - id.matrix[r][c]).abs() < 1e-5);
            }
        }
    }

    #[test]
    fn inverse_matrix_of_identity_is_identity() {
        let id = identity_mdxa_bone();
        let inv = inverse_matrix(&id);
        for r in 0..3 {
            for c in 0..4 {
                assert!((inv.matrix[r][c] - id.matrix[r][c]).abs() < 1e-6);
            }
        }
    }

    #[test]
    fn generate_world_matrix_at_zero_angles_carries_origin_translation() {
        let (world, world_inv) = g2_generate_world_matrix([0.0, 0.0, 0.0], [1.0, 2.0, 3.0]);
        assert_eq!(world.matrix[0][3], 1.0);
        assert_eq!(world.matrix[1][3], 2.0);
        assert_eq!(world.matrix[2][3], 3.0);
        // Inverting an untranslated-rotation + pure-translation matrix must
        // recover the negated origin in its own translation column.
        assert!((world_inv.matrix[0][3] - -1.0).abs() < 1e-5);
        assert!((world_inv.matrix[1][3] - -2.0).abs() < 1e-5);
        assert!((world_inv.matrix[2][3] - -3.0).abs() < 1e-5);
    }

    #[test]
    fn transform_and_translate_point_applies_identity_unchanged() {
        let id = identity_mdxa_bone();
        let p = transform_and_translate_point([1.0, 2.0, 3.0], &id);
        assert_eq!(p, [1.0, 2.0, 3.0]);
    }

    // --- G2_SegmentTriangleTest / G2_AreaOfTri / G2_BuildHitPointST ---------
    // Source: `oracle/codemp/ghoul2/G2_misc.cpp:665-777`

    #[test]
    fn segment_triangle_test_hits_a_triangle_face_on() {
        let a = [0.0, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0];
        let c = [0.0, 1.0, 0.0];
        let start = [0.1, 0.1, 1.0];
        let end = [0.1, 0.1, -1.0];
        let hit = g2_segment_triangle_test(start, end, a, b, c, true, true);
        assert!(hit.is_some());
        let (point, _normal, _denom) = hit.unwrap();
        assert!((point[2]).abs() < 1e-5);
    }

    #[test]
    fn segment_triangle_test_misses_when_ray_is_outside_the_triangle() {
        let a = [0.0, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0];
        let c = [0.0, 1.0, 0.0];
        let start = [5.0, 5.0, 1.0];
        let end = [5.0, 5.0, -1.0];
        assert!(g2_segment_triangle_test(start, end, a, b, c, true, true).is_none());
    }

    #[test]
    fn area_of_tri_is_twice_the_triangle_area() {
        // Right triangle with legs of length 2: area = 2, G2_AreaOfTri = 4.
        let area = g2_area_of_tri([0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]);
        assert!((area - 4.0).abs() < 1e-5);
    }

    #[test]
    fn build_hit_point_st_recovers_a_vertex_uv_at_that_vertex() {
        let a = [0.0, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0];
        let c = [0.0, 1.0, 0.0];
        let (s, t, _bi, _bj) = g2_build_hit_point_st(a, 0.0, 0.0, b, 1.0, 0.0, c, 0.0, 1.0, a);
        assert!((s - 0.0).abs() < 1e-4);
        assert!((t - 0.0).abs() < 1e-4);
    }

    // --- G2_DecideTraceLod ---------------------------------------------------
    // Source: `oracle/codemp/ghoul2/G2_misc.cpp:376-398`

    #[test]
    fn decide_trace_lod_clamps_to_lod_bias_and_model_lod_count() {
        // Synthetic mdxmHeader_t: only `numLODs` (offset 144) is populated;
        // every other field defaults to 0.
        let mut mdxm = vec![0u8; MDXM_HEADER_SIZE];
        mdxm[MDXM_OFS_NUM_LODS..MDXM_OFS_NUM_LODS + 4].copy_from_slice(&3i32.to_ne_bytes());
        let mut host = MockHost::new();
        host.mdxm_blocks.insert(1, mdxm);

        let mut info = CGhoul2Info::default();
        info.model = 1;
        info.lod_bias = 0;
        assert_eq!(g2_decide_trace_lod(&mut host, &info, 1), 1);
        // Above the model's LOD count (3) clamps to numLODs - 1.
        assert_eq!(g2_decide_trace_lod(&mut host, &info, 5), 2);

        info.lod_bias = 2;
        // `mLodBias` overrides a lower requested LOD.
        assert_eq!(g2_decide_trace_lod(&mut host, &info, 0), 2);
    }

    // --- register-model host gap (module-doc note; same shape as
    // `api_models.rs`'s identically-named helpers) ---------------------------

    #[test]
    fn register_model_diverges_via_host_error() {
        let mut host = MockHost::new();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            register_model(&mut host, "models/x.glm")
        }));
        assert!(result.is_err());
        assert_eq!(host.errors.len(), 1);
        assert_eq!(host.errors[0].0, errorParm_t::ERR_DROP);
    }

    #[test]
    fn setup_model_pointers_skips_registration_when_modelindex_is_unset() {
        // `modelindex == -1` (the `CGhoul2Info::default()` state) never
        // touches `register_model`/`register_server_model`, so this must NOT
        // panic even though the host has no real registration service.
        let mut host = MockHost::new();
        let mut info = CGhoul2Info::default();
        assert!(!g2_setup_model_pointers(&mut host, &mut info));
        assert!(!info.valid);
    }

    // --- save/load round trip (module doc comment: self-consistent Rust
    // round trip, not oracle-byte-compatible; graph-dead server-side per
    // `api_saveload.rs`'s own finding) ----------------------------------------

    #[test]
    fn save_then_load_empty_ghoul2_round_trips_to_zero_instances() {
        let mut g2 = Ghoul2System::default();
        let mut host = MockHost::new();
        let ghoul2 = CGhoul2Info_v { mItem: 0 };

        let saved = g2_save_ghoul2_models(&g2, &ghoul2).expect("always Some");
        assert_eq!(saved, vec![0u8, 0, 0, 0]);

        let mut loaded = CGhoul2Info_v { mItem: 0 };
        g2_load_ghoul2_model(&mut g2, &mut host, &mut loaded, &saved);
        assert_eq!(loaded.size(&g2), 0);
    }

    #[test]
    fn save_then_load_round_trips_middle_band_fields_with_no_model() {
        // `modelindex == -1` keeps this test off the `register_model` panic
        // path (see `setup_model_pointers_skips_registration...` above).
        let mut g2 = Ghoul2System::default();
        let mut host = MockHost::new();
        let mut ghoul2 = CGhoul2Info_v { mItem: 0 };
        ghoul2.push_back(
            &mut g2,
            CGhoul2Info {
                lod_bias: 7,
                surface_root: 2,
                flags: 0x40,
                ..CGhoul2Info::default()
            },
        );

        let saved = g2_save_ghoul2_models(&g2, &ghoul2).expect("always Some");

        let mut g2b = Ghoul2System::default();
        let mut loaded = CGhoul2Info_v { mItem: 0 };
        g2_load_ghoul2_model(&mut g2b, &mut host, &mut loaded, &saved);

        assert_eq!(loaded.size(&g2b), 1);
        let info = loaded.get(&g2b, 0);
        assert_eq!(info.lod_bias, 7);
        assert_eq!(info.surface_root, 2);
        assert_eq!(info.flags, 0x40);
    }
}
