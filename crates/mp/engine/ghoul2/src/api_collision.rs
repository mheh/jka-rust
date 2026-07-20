#![allow(non_camel_case_types, non_snake_case)]

//! `G2API` collision + time — the server collision-trace entry points, the
//! bolt-matrix-to-vector helper the ragdoll solver reuses, the `G2TimeBases`
//! clock pair, and the listen-server-opt client/server instance sync stub.
//!
//! Per `docs/subsystems/ghoul2-server.md` roster (`api_collision.rs`, class
//! "G2API collision+time"): `CollisionDetect`/`CollisionDetectCache`,
//! `GiveMeVectorFromMatrix`, `SetTime`/`GetTime` (`G2TimeBases`),
//! `OverrideServerWithClientData` (WinDed live arm -> `bool{false}`,
//! `G2SV-D4`).
//!
//! Every `G2API_*` entry keeps its 1:1 signature (`G2SV-D6`) and threads
//! `g2: &mut Ghoul2System` (ruling 4/11, state threaded not reached).
//! `g2api_collision_detect`/`g2api_set_time`/`g2api_get_time`/
//! `g2api_override_server_with_client_data` are frozen verbatim in the doc's
//! `## Seam definition`; see each function's doc comment for the rest.
//!
//! **Doc/oracle gaps found while transcribing this class (reported upstream,
//! not fixed here — porting-rules §17/CLAUDE.md "private helpers included"):**
//! 1. `G2API_CollisionDetectCache`'s private helper `static inline bool
//!    G2_NeedRetransform(CGhoul2Info *g2, int frameNum)` (`G2_API.cpp:2003-2031`,
//!    physically adjacent to `G2API_CollisionDetectCache` in the same TU) is
//!    named nowhere in the doc's roster summary or Method transcription table;
//!    ported here as `g2_need_retransform`, this file's own private helper,
//!    since it has no cross-file caller and no other rostered home.
//! 2. `g2api_collision_detect_cache`'s transformed-verts-array alloc path reads
//!    `mod->mdxm->numSurfaces` (`G2_API.cpp:2062`) the same way
//!    `api_surfaces.rs`'s `g2api_get_surface_name` already does — that file's
//!    own module doc-comment flags the `mdxmHeader_t` byte-offset table as an
//!    undocumented shape (no doc section spells it out); this file needs the
//!    same `numSurfaces` offset and re-derives it locally (no shared home
//!    exists yet to import it from), duplicating rather than inventing a new
//!    offset.

use mp_host_interface::EngineHost;
use mp_qshared::common::mp::qcommon::collision_record::MAX_G2_COLLISIONS;
use mp_qshared::shared::q_math::TransformAndTranslatePoint;
use mp_qshared::shared::{mdxaBone_t, vec3_t, CollisionRecord_t, Eorientations};

use crate::ghoul2_system::{Ghoul2System, NUM_G2T_TIME};
use crate::shared::cghoul2_info::CGhoul2Info;
use crate::shared::cghoul2_info_v::CGhoul2Info_v;

/// Raven `#define GHOUL2_ZONETRANSALLOC 0x2000` — the "transformed-verts
/// buffer is this instance's own zone alloc" flag on `CGhoul2Info::mFlags`
/// `g2api_collision_detect_cache`'s alloc path tests/sets. Not owned by any
/// single roster file (porting-rules §A1), defined locally where first
/// needed, matching `api_bones.rs`'s `GHOUL2_RAG_STARTED` precedent.
///
/// Source: `oracle/codemp/ghoul2/ghoul2_shared.h:235`
const GHOUL2_ZONETRANSALLOC: i32 = 0x2000;

/// Raven `#define BONE_ANIM_OVERRIDE_LOOP 0x0010` — read by
/// `g2_need_retransform` below.
///
/// Source: `oracle/codemp/ghoul2/G2.h:23`
const BONE_ANIM_OVERRIDE_LOOP: i32 = 0x0010;

/// Raven `#define BONE_NEED_TRANSFORM 0x8000` — read (and cleared) by
/// `g2_need_retransform` below.
///
/// Source: `oracle/codemp/ghoul2/G2.h:14`
const BONE_NEED_TRANSFORM: i32 = 0x8000;

// `mdxmHeader_t` layout (oracle/codemp/renderer/mdx_format.h:151-172): int
// ident, int version, char name[64], char animName[64], int animIndex, int
// numBones, int numLODs, int ofsLODs, int numSurfaces, int ofsSurfHierarchy,
// int ofsEnd — every field 4-byte-aligned with no padding, so `numSurfaces`
// sits at byte offset 152 (same derivation `api_surfaces.rs`'s
// `g2api_get_surface_name` already uses; module-doc gap note #2 above). This
// crate never names the `mdxm*` types (`G2SV-D5`); the offset below is the
// same raw byte arithmetic Raven itself does off the loader-owned block.
const NUM_SURFACES_OFFSET: usize = 152;

/// Raven `static void G2API_CollisionDetectCache(...)`'s private helper
/// `static inline bool G2_NeedRetransform(CGhoul2Info *g2, int frameNum)` —
/// walks `g2->mBlist`, deciding (and, as a side effect, clearing
/// `BONE_NEED_TRANSFORM` on) whether any bone's currently-lerped animation
/// frame demands a fresh skeleton/vert transform this call. Module-doc gap
/// note #1: undocumented private helper, ported here as this file's own.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:2003-2031`
fn g2_need_retransform(ghl_info: &mut CGhoul2Info, frame_num: i32) -> bool {
    let mut need_trans = false;
    for bone in ghl_info.blist.iter_mut() {
        let time = if bone.pauseTime != 0 {
            (bone.pauseTime - bone.startTime) as f32 / 50.0
        } else {
            (frame_num - bone.startTime) as f32 / 50.0
        };
        let new_frame = (bone.startFrame as f32 + time * bone.animSpeed) as i32;

        if new_frame < bone.endFrame
            || (bone.flags & BONE_ANIM_OVERRIDE_LOOP) != 0
            || (bone.flags & BONE_NEED_TRANSFORM) != 0
        {
            bone.flags &= !BONE_NEED_TRANSFORM;
            need_trans = true;
        }
    }
    need_trans
}

/// Raven `static char noSurface`-style shared sentinel — the "no collisions"
/// `CollisionRecord_t`: `mEntityNum = -1` is the empty-slot marker
/// `G2_TraceModels`/the counting loop below both key off.
fn empty_collision_record() -> CollisionRecord_t {
    CollisionRecord_t {
        mDistance: 0.0,
        mEntityNum: -1,
        mModelIndex: 0,
        mPolyIndex: 0,
        mSurfaceIndex: 0,
        mCollisionPosition: [0.0; 3],
        mCollisionNormal: [0.0; 3],
        mFlags: 0,
        mMaterial: 0,
        mLocation: 0,
        mBarycentricI: 0.0,
        mBarycentricJ: 0.0,
    }
}

/// Raven's shared collision-detect tail (`G2_API.cpp:2115-2119,2167-2171`):
/// `for (i = 0; i < MAX_G2_COLLISIONS && collRecMap[i].mEntityNum != -1; i++);`
/// then `qsort(collRecMap, i, sizeof(CollisionRecord_t), QsortDistance)`. Since
/// `G2_TraceModels` always fills the first free (`mEntityNum == -1`) slot
/// (`G2_misc.cpp:1114,1346`), the populated records are exactly the packed
/// prefix this counting loop finds; the doc's out-param -> `Vec` mapping
/// collapses that prefix into the owned return.
fn collect_sorted_collisions(
    coll_rec_map: &[CollisionRecord_t; MAX_G2_COLLISIONS],
) -> Vec<CollisionRecord_t> {
    let mut count = 0usize;
    while count < MAX_G2_COLLISIONS && coll_rec_map[count].mEntityNum != -1 {
        count += 1;
    }
    let mut result = coll_rec_map[..count].to_vec();
    // Raven's `QsortDistance` (`G2_API.cpp:1993-2000`) returns `1` even when
    // `mDistance` is equal (never `0`) — an inconsistent qsort comparator
    // (porting-rules §19, UB); `total_cmp` gives a well-defined ascending
    // order instead of reproducing the comparator's undefined tie-breaking.
    result.sort_by(|a, b| a.mDistance.total_cmp(&b.mDistance));
    result
}

/// Raven `void G2API_CollisionDetect(CollisionRecord_t *collRecMap, ...)` —
/// walk each model in `ghoul2`, build the skeleton/transform for `frameNumber`,
/// then trace `rayStart`/`rayEnd` (transformed to model space) against every
/// polygon via `G2_TraceModels`, distance-sorting the resulting collision
/// records. A no-op (empty result) when `G2_SetupModelPointers` fails.
///
/// Frozen verbatim in `docs/subsystems/ghoul2-server.md` `## Seam definition`:
/// the out-param `CollisionRecord_t *collRecMap` array becomes an owned
/// `Vec<CollisionRecord_t>` return (the populated `mEntityNum != -1` entries),
/// and the `CMiniHeap *G2VertSpace` scratch-heap parameter is dropped from the
/// signature (not threaded as a param, not a `Ghoul2System` field either — see
/// this file's module-level `problems` note, reported upstream).
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:2114-2145`
#[allow(clippy::too_many_arguments)]
pub fn g2api_collision_detect(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghoul2: &mut CGhoul2Info_v,
    angles: vec3_t,
    position: vec3_t,
    frame_number: i32,
    ent_num: i32,
    ray_start: vec3_t,
    ray_end: vec3_t,
    scale: vec3_t,
    trace_flags: i32,
    use_lod: i32,
    f_radius: f32,
) -> Vec<CollisionRecord_t> {
    if !crate::misc::g2_setup_model_pointers_v(g2, host, ghoul2) {
        return Vec::new();
    }

    // make sure we have transformed the whole skeletons for each model
    crate::render::skeleton::g2_construct_ghoul_skeleton(
        g2,
        host,
        ghoul2,
        frame_number,
        true,
        scale,
    );

    // pre generate the world matrix - used to transform the incoming ray
    let (world_matrix, world_matrix_inv) = crate::misc::g2_generate_world_matrix(angles, position);

    // Raven's `G2VertSpace->ResetHeap()` has no Rust-side effect here
    // (module-doc gap note #2 / `misc.rs` module-doc #4: `CMiniHeap` dropped
    // from every signature in this crate).
    crate::misc::g2_transform_model(g2, host, ghoul2, frame_number, scale, use_lod, false);

    // model is built. Lets check to see if any triangles are actually hit.
    // first up, translate the ray to model space
    let mut trans_ray_start = [0.0; 3];
    TransformAndTranslatePoint(ray_start, &mut trans_ray_start, &world_matrix_inv);
    let mut trans_ray_end = [0.0; 3];
    TransformAndTranslatePoint(ray_end, &mut trans_ray_end, &world_matrix_inv);

    // now walk each model and check the ray against each poly
    let mut coll_rec_map = [empty_collision_record(); MAX_G2_COLLISIONS];
    crate::misc::g2_trace_models(
        g2,
        host,
        ghoul2,
        trans_ray_start,
        trans_ray_end,
        &mut coll_rec_map,
        ent_num,
        trace_flags,
        use_lod,
        f_radius,
        0.0,
        0.0,
        0.0,
        None,
        None,
        false,
        &world_matrix,
    );

    // now sort the resulting array of collision records so they are distance ordered
    collect_sorted_collisions(&coll_rec_map)
}

/// Raven `void G2API_CollisionDetectCache(CollisionRecord_t *collRecMap, ...)`
/// — the cached twin of `G2API_CollisionDetect`: reuses each model's
/// `mTransformedVertsArray` across calls (only rebuilding the skeleton/
/// transform via `G2_NeedRetransform`/`G2API_GetTime`-gated staleness) before
/// running the same trace + distance sort.
///
/// Not spelled verbatim in the doc's `## Seam definition` code block (only
/// `g2api_collision_detect` is illustrated there), but the roster prose pairs
/// it with `CollisionDetect` 1:1 and the oracle declares it with the identical
/// parameter list (`G2_local.h:154-155`) minus the same dropped
/// `CMiniHeap *G2VertSpace`. Transcribed here by the same out-param → owned
/// `Vec<CollisionRecord_t>` mapping `g2api_collision_detect` freezes, per
/// `G2SV-D6` 1:1 arity; reported upstream as a doc completeness gap, not a
/// wrong signature (`problems` note).
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:2035-2113`
#[allow(clippy::too_many_arguments)]
pub fn g2api_collision_detect_cache(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghoul2: &mut CGhoul2Info_v,
    angles: vec3_t,
    position: vec3_t,
    frame_number: i32,
    ent_num: i32,
    ray_start: vec3_t,
    ray_end: vec3_t,
    scale: vec3_t,
    trace_flags: i32,
    use_lod: i32,
    f_radius: f32,
) -> Vec<CollisionRecord_t> {
    // Raven: `int *test = ghoul2[0].mTransformedVertsArray;` — a dead local
    // (never read again) dropped here; on an empty `ghoul2` handle it would
    // also be an out-of-bounds `operator[]` read in the oracle's release
    // (`NDEBUG`, asserts compiled out) build (porting-rules §19: UB, not
    // reproduced).
    if !crate::misc::g2_setup_model_pointers_v(g2, host, ghoul2) {
        return Vec::new();
    }

    let t_frame_num = g2api_get_time(g2, frame_number);

    // make sure we have transformed the whole skeletons for each model
    let need_retransform = g2_need_retransform(ghoul2.get_mut(g2, 0), t_frame_num);
    let no_transformed_verts = ghoul2.get(g2, 0).transformed_verts_array.is_none();

    if need_retransform || no_transformed_verts {
        // optimization, only create new transform space if we need to,
        // otherwise store it off!
        for idx in 0..ghoul2.size(g2) {
            let model = ghoul2.get(g2, idx).model;
            let mdxm = host.model_mdxm(model);
            let num_surfaces = unsafe { *(mdxm.byte_add(NUM_SURFACES_OFFSET) as *const i32) };

            let info = ghoul2.get_mut(g2, idx);
            if info.transformed_verts_array.is_none() || (info.flags & GHOUL2_ZONETRANSALLOC) == 0 {
                // reworked so we only alloc once! Raven `Z_Malloc(iSize,
                // TAG_GHOUL2, qtrue)` — `qtrue` zeroes the block.
                info.transformed_verts_array = Some(vec![0i32; num_surfaces as usize]);
            }
            info.flags |= GHOUL2_ZONETRANSALLOC;
        }

        crate::render::skeleton::g2_construct_ghoul_skeleton(
            g2,
            host,
            ghoul2,
            frame_number,
            true,
            scale,
        );
        // Raven's `G2VertSpace->ResetHeap()` has no Rust-side effect here
        // (module-doc gap note #2 / `misc.rs` module-doc #4).

        // now having done that, time to build the model
        crate::misc::g2_transform_model(g2, host, ghoul2, frame_number, scale, use_lod, false);
    }

    // pre generate the world matrix - used to transform the incoming ray
    let (world_matrix, world_matrix_inv) = crate::misc::g2_generate_world_matrix(angles, position);

    // model is built. Lets check to see if any triangles are actually hit.
    // first up, translate the ray to model space
    let mut trans_ray_start = [0.0; 3];
    TransformAndTranslatePoint(ray_start, &mut trans_ray_start, &world_matrix_inv);
    let mut trans_ray_end = [0.0; 3];
    TransformAndTranslatePoint(ray_end, &mut trans_ray_end, &world_matrix_inv);

    // now walk each model and check the ray against each poly
    let mut coll_rec_map = [empty_collision_record(); MAX_G2_COLLISIONS];
    crate::misc::g2_trace_models(
        g2,
        host,
        ghoul2,
        trans_ray_start,
        trans_ray_end,
        &mut coll_rec_map,
        ent_num,
        trace_flags,
        use_lod,
        f_radius,
        0.0,
        0.0,
        0.0,
        None,
        None,
        false,
        &world_matrix,
    );

    // now sort the resulting array of collision records so they are distance ordered
    collect_sorted_collisions(&coll_rec_map)
}

/// Raven `void G2API_GiveMeVectorFromMatrix(mdxaBone_t *boltMatrix,
/// Eorientations flags, vec3_t vec)` — read one axis column (or its negation)
/// out of `boltMatrix` per `flags` into `vec`; a pure computation with no
/// `Ghoul2System`/`EngineHost` access and no failure path (every `Eorientations`
/// enumerator is handled by the oracle `switch`).
///
/// Not spelled verbatim in the doc's `## Seam definition` code block (only the
/// roster prose names it). Per §C7's out-param → return default (this
/// function has no bool/qboolean discriminator to classify against — it is
/// `void` and unconditionally writes on every reachable `flags` value), the
/// out-param `vec` becomes a returned `vec3_t` rather than a `&mut` write-
/// through parameter. No `g2`/`host` thread: the function reads only its
/// value arguments, matching the sibling free-fn helpers in `## Seam
/// definition` (`multiply_3x4_matrix`, `g2_transform_bone`) that also omit
/// them. Reported upstream as a doc completeness gap, not a wrong signature
/// (`problems` note); also server-live via the ragdoll solver
/// (`G2_bones.cpp:3411` et al., `ragdoll.rs`), not just the (absent) trap
/// surface.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:2196-2232`
pub fn g2api_give_me_vector_from_matrix(bolt_matrix: &mdxaBone_t, flags: Eorientations) -> vec3_t {
    match flags {
        Eorientations::ORIGIN => [
            bolt_matrix.matrix[0][3],
            bolt_matrix.matrix[1][3],
            bolt_matrix.matrix[2][3],
        ],
        Eorientations::POSITIVE_Y => [
            bolt_matrix.matrix[0][1],
            bolt_matrix.matrix[1][1],
            bolt_matrix.matrix[2][1],
        ],
        Eorientations::POSITIVE_X => [
            bolt_matrix.matrix[0][0],
            bolt_matrix.matrix[1][0],
            bolt_matrix.matrix[2][0],
        ],
        Eorientations::POSITIVE_Z => [
            bolt_matrix.matrix[0][2],
            bolt_matrix.matrix[1][2],
            bolt_matrix.matrix[2][2],
        ],
        Eorientations::NEGATIVE_Y => [
            -bolt_matrix.matrix[0][1],
            -bolt_matrix.matrix[1][1],
            -bolt_matrix.matrix[2][1],
        ],
        Eorientations::NEGATIVE_X => [
            -bolt_matrix.matrix[0][0],
            -bolt_matrix.matrix[1][0],
            -bolt_matrix.matrix[2][0],
        ],
        Eorientations::NEGATIVE_Z => [
            -bolt_matrix.matrix[0][2],
            -bolt_matrix.matrix[1][2],
            -bolt_matrix.matrix[2][2],
        ],
    }
}

/// Raven `void G2API_SetTime(int currentTime, int clock)` — write
/// `G2TimeBases[clock]`, then reset `G2TimeBases[1]` to `0` (fall back to
/// server time) if the client clock has drifted more than 200ms ahead of the
/// server clock.
///
/// Frozen verbatim in `docs/subsystems/ghoul2-server.md` `## Seam definition`.
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:162-177`
pub fn g2api_set_time(g2: &mut Ghoul2System, current_time: i32, clock: i32) {
    // Raven: `assert(clock>=0&&clock<NUM_G2T_TIME);` (compiled out, `NDEBUG`).
    debug_assert!(clock >= 0 && (clock as usize) < NUM_G2T_TIME);
    g2.time_bases[clock as usize] = current_time;
    if g2.time_bases[1] > g2.time_bases[0] + 200 {
        g2.time_bases[1] = 0; // use server time instead
    }
}

/// Raven `int G2API_GetTime(int argTime)` — return `G2TimeBases[1]` (client
/// clock) if nonzero, else fall back to `G2TimeBases[0]` (server clock);
/// `argTime` itself is unread (oracle comment: "this may or may not return arg
/// depending on ghoul2_time cvar").
///
/// Frozen verbatim in `docs/subsystems/ghoul2-server.md` `## Seam definition`
/// (immutable `&Ghoul2System` — a pure read of `time_bases`).
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:179-188`
pub fn g2api_get_time(g2: &Ghoul2System, arg_time: i32) -> i32 {
    let _ = arg_time;
    let ret = g2.time_bases[1];
    if ret != 0 {
        ret
    } else {
        g2.time_bases[0]
    }
}

/// Raven `qboolean G2API_OverrideServerWithClientData(CGhoul2Info
/// *serverInstance)` — copy the attached client-side instance's transformed
/// bone cache onto the server instance so the server can reuse client work.
/// WinDed builds `_G2_LISTEN_SERVER_OPT` OFF (`G2SV-D4`), so the live arm is
/// unconditionally `return qfalse` before any of the listen-server body
/// (`g2ClientAttachments`/`mSkelFrameNum`/`mBoneCache` reads) runs;
/// `serverInstance` is unread on that path.
///
/// Frozen verbatim in `docs/subsystems/ghoul2-server.md` `## Seam definition`:
/// takes the single `CGhoul2Info *serverInstance` (`=&g2[0]`,
/// `sv_game.cpp:1599`), NOT the `CGhoul2Info_v` wrapper (1:1 arity, `G2SV-D6`).
/// `g2` is threaded per ruling 11 though the WinDed live arm never reads it.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:239-282`
pub fn g2api_override_server_with_client_data(
    g2: &mut Ghoul2System,
    server_instance: &mut CGhoul2Info,
) -> bool {
    let _ = (g2, server_instance);
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- g2_need_retransform -------------------------------------------------
    // Source: `oracle/codemp/ghoul2/G2_API.cpp:2003-2031`

    fn bone_with(
        start_frame: i32,
        end_frame: i32,
        start_time: i32,
        pause_time: i32,
        anim_speed: f32,
        flags: i32,
    ) -> crate::shared::bone_info_t::boneInfo_t {
        crate::shared::bone_info_t::boneInfo_t {
            startFrame: start_frame,
            endFrame: end_frame,
            startTime: start_time,
            pauseTime: pause_time,
            animSpeed: anim_speed,
            flags,
            ..unsafe { core::mem::zeroed() }
        }
    }

    #[test]
    fn need_retransform_false_when_animation_reached_end_frame_and_not_looping() {
        // time = (100-0)/50 = 2.0; newFrame = 0 + 2.0*1.0 = 2, not < endFrame (2).
        let mut info = CGhoul2Info::default();
        info.blist.push(bone_with(0, 2, 0, 0, 1.0, 0));
        assert!(!g2_need_retransform(&mut info, 100));
    }

    #[test]
    fn need_retransform_true_while_animation_still_playing() {
        // time = (25-0)/50 = 0.5; newFrame = 0 + 0.5*1.0 = 0, < endFrame (2).
        let mut info = CGhoul2Info::default();
        info.blist.push(bone_with(0, 2, 0, 0, 1.0, 0));
        assert!(g2_need_retransform(&mut info, 25));
    }

    #[test]
    fn need_retransform_uses_pause_time_when_paused() {
        // paused at time 10, so elapsed is fixed at (10-0)/50=0.2 regardless of
        // frameNum; newFrame = 0, < endFrame (2) -> still needs transform.
        let mut info = CGhoul2Info::default();
        info.blist.push(bone_with(0, 2, 0, 10, 1.0, 0));
        assert!(g2_need_retransform(&mut info, 999_999));
    }

    #[test]
    fn need_retransform_true_and_clears_bone_need_transform_flag() {
        let mut info = CGhoul2Info::default();
        info.blist
            .push(bone_with(0, 2, 0, 0, 1.0, BONE_NEED_TRANSFORM));
        assert!(g2_need_retransform(&mut info, 100));
        assert_eq!(info.blist[0].flags & BONE_NEED_TRANSFORM, 0);
    }

    #[test]
    fn need_retransform_true_when_override_loop_set_past_end() {
        let mut info = CGhoul2Info::default();
        info.blist
            .push(bone_with(0, 2, 0, 0, 1.0, BONE_ANIM_OVERRIDE_LOOP));
        assert!(g2_need_retransform(&mut info, 100));
    }

    // --- g2api_give_me_vector_from_matrix ------------------------------------
    // Source: `oracle/codemp/ghoul2/G2_API.cpp:2196-2232`

    fn sample_matrix() -> mdxaBone_t {
        mdxaBone_t {
            matrix: [
                [1.0, 2.0, 3.0, 4.0],
                [5.0, 6.0, 7.0, 8.0],
                [9.0, 10.0, 11.0, 12.0],
            ],
        }
    }

    #[test]
    fn give_me_vector_origin_reads_translation_column() {
        let m = sample_matrix();
        assert_eq!(
            g2api_give_me_vector_from_matrix(&m, Eorientations::ORIGIN),
            [4.0, 8.0, 12.0]
        );
    }

    #[test]
    fn give_me_vector_positive_x_reads_column_zero() {
        let m = sample_matrix();
        assert_eq!(
            g2api_give_me_vector_from_matrix(&m, Eorientations::POSITIVE_X),
            [1.0, 5.0, 9.0]
        );
    }

    #[test]
    fn give_me_vector_negative_x_negates_column_zero() {
        let m = sample_matrix();
        assert_eq!(
            g2api_give_me_vector_from_matrix(&m, Eorientations::NEGATIVE_X),
            [-1.0, -5.0, -9.0]
        );
    }

    #[test]
    fn give_me_vector_positive_and_negative_z_are_negations() {
        let m = sample_matrix();
        let pos = g2api_give_me_vector_from_matrix(&m, Eorientations::POSITIVE_Z);
        let neg = g2api_give_me_vector_from_matrix(&m, Eorientations::NEGATIVE_Z);
        assert_eq!(pos, [3.0, 7.0, 11.0]);
        assert_eq!(neg, [-3.0, -7.0, -11.0]);
    }

    // --- g2api_set_time / g2api_get_time -------------------------------------
    // Source: `oracle/codemp/ghoul2/G2_API.cpp:162-188`

    #[test]
    fn get_time_falls_back_to_server_clock_when_client_clock_zero() {
        let mut g2 = Ghoul2System::default();
        g2api_set_time(&mut g2, 500, 0); // server clock
        assert_eq!(g2api_get_time(&g2, 0), 500);
    }

    #[test]
    fn get_time_prefers_client_clock_when_set() {
        let mut g2 = Ghoul2System::default();
        g2api_set_time(&mut g2, 500, 0); // server clock
        g2api_set_time(&mut g2, 550, 1); // client clock, within 200ms of server
        assert_eq!(g2api_get_time(&g2, 0), 550);
    }

    #[test]
    fn set_time_resets_client_clock_when_drifted_past_server_by_200ms() {
        let mut g2 = Ghoul2System::default();
        g2api_set_time(&mut g2, 500, 0); // server clock
        g2api_set_time(&mut g2, 701, 1); // client clock drifts 201ms ahead
        assert_eq!(g2.time_bases[1], 0);
        assert_eq!(g2api_get_time(&g2, 0), 500);
    }

    // --- g2api_override_server_with_client_data ------------------------------
    // Source: `oracle/codemp/ghoul2/G2_API.cpp:239-282`

    #[test]
    fn override_server_with_client_data_is_always_false_under_windeed() {
        let mut g2 = Ghoul2System::default();
        let mut server_instance = CGhoul2Info::default();
        assert!(!g2api_override_server_with_client_data(
            &mut g2,
            &mut server_instance
        ));
    }
}
