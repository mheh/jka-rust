//! `G2API` bones — the non-ragdoll bone-anim/bone-angles `G2API_*` wrappers
//! over the internal `G2_Bones.cpp` bone-list logic.
//!
//! Per `docs/subsystems/ghoul2-server.md` roster (`api_bones.rs`, class "G2API
//! bones"): `SetBoneAnim`/`GetBoneAnim`/`GetAnimRange`/`PauseBoneAnim`/
//! `StopBoneAnim`/`SetBoneAngles`(`+Matrix`,`+Index`)/`RemoveBone`/
//! `GetBoneIndex`/`DoesBoneExist`/`AnimateG2Models` wrappers over `G2_bones`
//! internals (the `bones.rs` roster file, separately ported).
//!
//! Every `G2API_*` entry keeps its 1:1 signature (`G2SV-D6`) and threads
//! `g2: &mut Ghoul2System` (ruling 4/11, state threaded not reached); per the
//! `## Slice hooks` "Thin wrappers" classification this file's bodies open
//! with `G2_SetupModelPointers` (`G2_misc.cpp:1839`) — a loader model-memory
//! read — so each also threads `host: &mut impl EngineHost`, **except**
//! `g2api_set_bone_anim`, whose signature is quoted **verbatim** (no `host`)
//! in the doc's `## Seam definition` — see that function's doc comment for
//! the resulting inconsistency, reported upstream.
//!
//! Out-param classification follows the frozen discriminator ("Out-param
//! contract for the un-illustrated `G2API_*` functions", `G2SV-D1`
//! generalized): a failure path that still writes its out-param keeps
//! `&mut T` + `bool`; a failure path that returns before touching it maps to
//! `Option<T>`. `g2api_get_bone_anim`/`g2api_get_anim_range` are the doc's own
//! named archetypes of the latter (`G2_API.cpp:1140,1191`: `return qfalse`
//! before any out-param write).

use core::mem;

use mp_host_interface::EngineHost;
use mp_qshared::shared::{mdxaBone_t, qhandle_t, vec3_t, Eorientations};

use crate::ghoul2_system::Ghoul2System;
use mp_host_interface::mdx::mdxa::MdxaView;
use crate::shared::cghoul2_info::CGhoul2Info;
use crate::shared::cghoul2_info_v::CGhoul2Info_v;

/// Raven `#define GHOUL2_RAG_STARTED 0x0010` — the "already a live ragdoll"
/// flag on `CGhoul2Info::mFlags` every `Set_Bone_Anim`/`Set_Bone_Angles`
/// wrapper below rejects. Not owned by any single roster file, so it is
/// defined locally where it is first needed (porting-rules §A1); the sibling
/// `api_ragdoll.rs`/`api_bolts.rs`/`api_models.rs` doc comments name the same
/// macro without a Rust binding yet.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:992`
const GHOUL2_RAG_STARTED: i32 = 0x0010;

/// Raven `G2API_SetBoneAnim` — clamp the anim range/`setFrame` inputs to
/// sane bounds, then forward to `G2_Set_Bone_Anim` on `ghoul2[modelIndex]`;
/// `qfalse` on an out-of-range `modelIndex` or a ragdoll-started instance.
///
/// **Frozen verbatim** in `docs/subsystems/ghoul2-server.md` `## Seam
/// definition` **without** a `host` parameter, even though the body opens
/// with `G2_SetupModelPointers` (`:1112`) exactly like `g2api_set_bone_angles`
/// below (which the same doc *does* thread `host` through). This is a
/// same-shape sibling inconsistency in the frozen doc, reported upstream
/// rather than silently added here (porting-rules §F: pinned signatures are
/// LAW, not improvised around).
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:1066-1138`
#[allow(clippy::too_many_arguments)]
pub fn g2api_set_bone_anim(
    g2: &mut Ghoul2System,
    ghoul2: &mut CGhoul2Info_v,
    model_index: i32,
    bone_name: &str,
    start_frame: i32,
    end_frame: i32,
    flags: i32,
    anim_speed: f32,
    current_time: i32,
    set_frame: f32,
    blend_time: i32,
) -> bool {
    // clamp the anim range/setFrame inputs (G2_API.cpp:1069-1093), independent of setup/model state
    let mut end_frame = end_frame;
    let mut start_frame = start_frame;
    let mut set_frame = set_frame;
    if end_frame <= 0 {
        end_frame = 1;
    }
    if end_frame >= 100_000 {
        end_frame = 1;
    }
    if start_frame < 0 {
        start_frame = 0;
    }
    if start_frame >= 100_000 {
        start_frame = 0;
    }
    if set_frame < 0.0 && set_frame != -1.0 {
        set_frame = 0.0;
    }
    if set_frame > 100_000.0 {
        set_frame = 0.0;
    }

    if ghoul2.size(g2) <= model_index {
        return false;
    }
    let ghl_info = ghoul2.get_mut(g2, model_index);

    // NOTE (reported upstream, see this fn's doc comment above): the frozen
    // `## Seam definition` signature carries no `host`, so `G2_SetupModelPointers`
    // (G2_API.cpp:1076) cannot be called here; this uses the instance's already-
    // cached `valid` flag from the last successful setup instead of revalidating.
    if !ghl_info.valid {
        return false;
    }
    if ghl_info.flags & GHOUL2_RAG_STARTED != 0 {
        return false;
    }

    ghl_info.skel_frame_num = 0;
    let mut blist = mem::take(&mut ghl_info.blist);
    let result = crate::bones::g2_set_bone_anim(
        ghl_info,
        &mut blist,
        bone_name,
        start_frame,
        end_frame,
        flags,
        anim_speed,
        current_time,
        set_frame,
        blend_time,
    );
    ghl_info.blist = blist;
    result
}

/// Raven `G2API_GetBoneAnim` — write-on-success-only (`G2SV-D1` generalized
/// discriminator's named archetype): `return qfalse` on
/// `G2_SetupModelPointers` failure **before** touching any out-param
/// (`currentFrame`/`startFrame`/`endFrame`/`flags`/`animSpeed`), so this maps
/// to `Option` (`None` = the untouched `qfalse` path), not a write-through
/// `&mut` out-param.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:1140-1189`
#[allow(clippy::too_many_arguments)]
pub fn g2api_get_bone_anim(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghl_info: &mut CGhoul2Info,
    bone_name: &str,
    current_time: i32,
    model_list: &[qhandle_t],
) -> Option<(f32, i32, i32, i32, f32)> {
    // Tuple order mirrors the Raven out-params: (currentFrame, startFrame,
    // endFrame, flags, animSpeed).
    if !crate::misc::g2_setup_model_pointers(host, ghl_info) {
        return None;
    }
    let a_current_time = crate::api_collision::g2api_get_time(g2, current_time);
    let model_index = ghl_info.modelindex;

    let mut current_frame = 0.0f32;
    let mut start_frame = 0i32;
    let mut end_frame = 0i32;
    let mut flags = 0i32;
    let mut ret_anim_speed = 0.0f32;

    let mut blist = mem::take(&mut ghl_info.blist);
    let ok = crate::bones::g2_get_bone_anim(
        ghl_info,
        &mut blist,
        bone_name,
        a_current_time,
        &mut current_frame,
        &mut start_frame,
        &mut end_frame,
        &mut flags,
        &mut ret_anim_speed,
        model_list,
        model_index,
    );
    ghl_info.blist = blist;

    if ok {
        Some((current_frame, start_frame, end_frame, flags, ret_anim_speed))
    } else {
        None
    }
}

/// Raven `G2API_GetAnimRange` — write-on-success-only, the `G2SV-D1`
/// discriminator's other named archetype: `return qfalse` before touching
/// `startFrame`/`endFrame` on `G2_SetupModelPointers` failure.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:1191-1220`
pub fn g2api_get_anim_range(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghl_info: &mut CGhoul2Info,
    bone_name: &str,
) -> Option<(i32, i32)> {
    // Tuple order: (startFrame, endFrame).
    let _ = g2;
    if !crate::misc::g2_setup_model_pointers(host, ghl_info) {
        return None;
    }
    let mut start_frame = 0i32;
    let mut end_frame = 0i32;
    let ok = crate::bones::g2_get_bone_anim_range(
        ghl_info,
        &ghl_info.blist,
        bone_name,
        &mut start_frame,
        &mut end_frame,
    );
    if ok {
        Some((start_frame, end_frame))
    } else {
        None
    }
}

/// Raven `G2API_PauseBoneAnim` — `qfalse` on `G2_SetupModelPointers`
/// failure, else `G2_Pause_Bone_Anim`'s result.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:1225-1231`
pub fn g2api_pause_bone_anim(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghl_info: &mut CGhoul2Info,
    bone_name: &str,
    current_time: i32,
) -> bool {
    let _ = g2;
    if !crate::misc::g2_setup_model_pointers(host, ghl_info) {
        return false;
    }
    let mut blist = mem::take(&mut ghl_info.blist);
    let result = crate::bones::g2_pause_bone_anim(ghl_info, &mut blist, bone_name, current_time);
    ghl_info.blist = blist;
    result
}

/// Raven `qboolean G2API_IsPaused(CGhoul2Info *ghlInfo, const char *boneName)`
/// — `qfalse` on `G2_SetupModelPointers` failure, else `G2_IsPaused`'s result
/// (internal `g2_is_paused` in `bones.rs`).
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:1234-1241`
pub fn g2api_is_paused(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghl_info: &mut CGhoul2Info,
    bone_name: &str,
) -> bool {
    let _ = g2;
    if !crate::misc::g2_setup_model_pointers(host, ghl_info) {
        return false;
    }
    crate::bones::g2_is_paused(host, &ghl_info.file_name, &ghl_info.blist, bone_name)
}

/// Raven `G2API_StopBoneAnim` — `qfalse` on `G2_SetupModelPointers`
/// failure, else `G2_Stop_Bone_Anim`'s result.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:1252-1259`
pub fn g2api_stop_bone_anim(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghl_info: &mut CGhoul2Info,
    bone_name: &str,
) -> bool {
    let _ = g2;
    if !crate::misc::g2_setup_model_pointers(host, ghl_info) {
        return false;
    }
    crate::bones::g2_stop_bone_anim(host, &ghl_info.file_name, &mut ghl_info.blist, bone_name)
}

/// Raven `qboolean G2API_StopBoneAnimIndex(CGhoul2Info *ghlInfo, const int
/// index)` — `qfalse` on `G2_SetupModelPointers` failure, else
/// `G2_Stop_Bone_Anim_Index`'s result (internal `g2_stop_bone_anim_index` in
/// `bones.rs`).
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:1243-1250`
pub fn g2api_stop_bone_anim_index(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghl_info: &mut CGhoul2Info,
    index: i32,
) -> bool {
    let _ = g2;
    if !crate::misc::g2_setup_model_pointers(host, ghl_info) {
        return false;
    }
    crate::bones::g2_stop_bone_anim_index(&mut ghl_info.blist, index)
}

/// Raven `qboolean G2API_SetBoneAngles(CGhoul2Info_v &ghoul2, ...)` —
/// bounds-checks `modelIndex`, flushes `mSkelFrameNum`, and forwards to
/// `G2_Set_Bone_Angles`; `qfalse` on an out-of-range `modelIndex`,
/// `G2_SetupModelPointers` failure, or a ragdoll-started instance.
///
/// **Frozen verbatim** in `docs/subsystems/ghoul2-server.md` `## Seam
/// definition` (with `host`, unlike the sibling `g2api_set_bone_anim` above).
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:1298-1334`
#[allow(clippy::too_many_arguments)]
pub fn g2api_set_bone_angles(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghoul2: &mut CGhoul2Info_v,
    model_index: i32,
    bone_name: &str,
    angles: vec3_t,
    flags: i32,
    up: Eorientations,
    left: Eorientations,
    forward: Eorientations,
    model_list: &[qhandle_t],
    blend_time: i32,
    current_time: i32,
) -> bool {
    if ghoul2.size(g2) <= model_index {
        return false;
    }
    let ghl_info = ghoul2.get_mut(g2, model_index);

    if !crate::misc::g2_setup_model_pointers(host, ghl_info) {
        return false;
    }
    if ghl_info.flags & GHOUL2_RAG_STARTED != 0 {
        return false;
    }

    ghl_info.skel_frame_num = 0;
    let model_index_field = ghl_info.modelindex;
    let mut blist = mem::take(&mut ghl_info.blist);
    let result = crate::bones::g2_set_bone_angles(
        ghl_info,
        &mut blist,
        bone_name,
        angles,
        flags,
        up,
        left,
        forward,
        model_list,
        model_index_field,
        blend_time,
        current_time,
    );
    ghl_info.blist = blist;
    result
}

/// Raven `G2API_SetBoneAnglesMatrix` — flushes `mSkelFrameNum`, forwards to
/// `G2_Set_Bone_Angles_Matrix`; `qfalse` on `G2_SetupModelPointers` failure.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:1351-1361`
#[allow(clippy::too_many_arguments)]
pub fn g2api_set_bone_angles_matrix(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghl_info: &mut CGhoul2Info,
    bone_name: &str,
    matrix: &mdxaBone_t,
    flags: i32,
    model_list: &[qhandle_t],
    blend_time: i32,
    current_time: i32,
) -> bool {
    let _ = g2;
    if !crate::misc::g2_setup_model_pointers(host, ghl_info) {
        return false;
    }
    ghl_info.skel_frame_num = 0;
    let model_index = ghl_info.modelindex;
    crate::bones::g2_set_bone_angles_matrix(
        host,
        &ghl_info.file_name,
        &mut ghl_info.blist,
        bone_name,
        matrix,
        flags,
        model_list,
        model_index,
        blend_time,
        current_time,
    )
}

/// Raven `G2API_SetBoneAnglesIndex` — flushes `mSkelFrameNum`, forwards to
/// `G2_Set_Bone_Angles_Index`; `qfalse` on `G2_SetupModelPointers` failure.
///
/// `yaw`/`pitch`/`roll` are declared `const int` in the header prototype
/// (`G2_local.h:170-172`) but `const Eorientations` in the `.cpp` definition
/// (`:1261-1263`) — a header/definition type mismatch inside the oracle
/// itself, not a doc/port divergence. The `.cpp` definition (ground truth for
/// behavior) is transcribed here; reported upstream as an oracle-source
/// quirk, not a doc mismatch.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:1261-1296`
#[allow(clippy::too_many_arguments)]
pub fn g2api_set_bone_angles_index(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghl_info: &mut CGhoul2Info,
    index: i32,
    angles: vec3_t,
    flags: i32,
    yaw: Eorientations,
    pitch: Eorientations,
    roll: Eorientations,
    model_list: &[qhandle_t],
    blend_time: i32,
    current_time: i32,
) -> bool {
    let _ = g2;
    if !crate::misc::g2_setup_model_pointers(host, ghl_info) {
        return false;
    }
    if ghl_info.flags & GHOUL2_RAG_STARTED != 0 {
        return false;
    }
    ghl_info.skel_frame_num = 0;
    let model_index = ghl_info.modelindex;
    crate::bones::g2_set_bone_angles_index(
        &mut ghl_info.blist,
        index,
        angles,
        flags,
        yaw,
        pitch,
        roll,
        model_list,
        model_index,
        blend_time,
        current_time,
    )
}

/// Raven `qboolean G2API_SetBoneAnglesMatrixIndex(CGhoul2Info *ghlInfo, const
/// int index, const mdxaBone_t &matrix, const int flags, qhandle_t *modelList,
/// int blendTime, int currentTime)` — flushes `mSkelFrameNum`, forwards to
/// `G2_Set_Bone_Angles_Matrix_Index` (internal
/// `g2_set_bone_angles_matrix_index` in `bones.rs`); `qfalse` on
/// `G2_SetupModelPointers` failure.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:1339-1350`
#[allow(clippy::too_many_arguments)]
pub fn g2api_set_bone_angles_matrix_index(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghl_info: &mut CGhoul2Info,
    index: i32,
    matrix: &mdxaBone_t,
    flags: i32,
    model_list: &[qhandle_t],
    blend_time: i32,
    current_time: i32,
) -> bool {
    let _ = g2;
    if !crate::misc::g2_setup_model_pointers(host, ghl_info) {
        return false;
    }
    ghl_info.skel_frame_num = 0;
    let model_index = ghl_info.modelindex;
    crate::bones::g2_set_bone_angles_matrix_index(
        &mut ghl_info.blist,
        index,
        matrix,
        flags,
        model_list,
        model_index,
        blend_time,
        current_time,
    )
}

/// Raven `qboolean G2API_StopBoneAnglesIndex(CGhoul2Info *ghlInfo, const int
/// index)` — flushes `mSkelFrameNum`, forwards to `G2_Stop_Bone_Angles_Index`
/// (internal `g2_stop_bone_angles_index` in `bones.rs`); `qfalse` on
/// `G2_SetupModelPointers` failure.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:1363-1372`
pub fn g2api_stop_bone_angles_index(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghl_info: &mut CGhoul2Info,
    index: i32,
) -> bool {
    let _ = g2;
    if !crate::misc::g2_setup_model_pointers(host, ghl_info) {
        return false;
    }
    ghl_info.skel_frame_num = 0;
    crate::bones::g2_stop_bone_angles_index(&mut ghl_info.blist, index)
}

/// Raven `qboolean G2API_StopBoneAngles(CGhoul2Info *ghlInfo, const char
/// *boneName)` — flushes `mSkelFrameNum`, forwards to `G2_Stop_Bone_Angles`
/// (internal `g2_stop_bone_angles` in `bones.rs`); `qfalse` on
/// `G2_SetupModelPointers` failure.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:1374-1384`
pub fn g2api_stop_bone_angles(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghl_info: &mut CGhoul2Info,
    bone_name: &str,
) -> bool {
    let _ = g2;
    if !crate::misc::g2_setup_model_pointers(host, ghl_info) {
        return false;
    }
    ghl_info.skel_frame_num = 0;
    crate::bones::g2_stop_bone_angles(host, &ghl_info.file_name, &mut ghl_info.blist, bone_name)
}

/// Raven `qboolean G2API_SetBoneAnimIndex(CGhoul2Info *ghlInfo, const int
/// index, const int startFrame, const int endFrame, const int flags, const
/// float animSpeed, const int currentTime, const float setFrame, const int
/// blendTime)` — clamps the anim range/`setFrame` inputs to sane bounds (same
/// clamp block as `g2api_set_bone_anim`), flushes `mSkelFrameNum`, then
/// forwards to `G2_Set_Bone_Anim_Index` (internal `g2_set_bone_anim_index` in
/// `bones.rs`); `qfalse` on `G2_SetupModelPointers` failure or a
/// ragdoll-started instance.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:996-1062`
#[allow(clippy::too_many_arguments)]
pub fn g2api_set_bone_anim_index(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghl_info: &mut CGhoul2Info,
    index: i32,
    start_frame: i32,
    end_frame: i32,
    flags: i32,
    anim_speed: f32,
    current_time: i32,
    set_frame: f32,
    blend_time: i32,
) -> bool {
    let _ = g2;
    let res = crate::misc::g2_setup_model_pointers(host, ghl_info);
    if res && (ghl_info.flags & GHOUL2_RAG_STARTED != 0) {
        return false;
    }

    // clamp the anim range/setFrame inputs (G2_API.cpp:1030-1054), independent of `res`
    let mut end_frame = end_frame;
    let mut start_frame = start_frame;
    let mut set_frame = set_frame;
    if end_frame <= 0 {
        end_frame = 1;
    }
    if end_frame >= 100_000 {
        end_frame = 1;
    }
    if start_frame < 0 {
        start_frame = 0;
    }
    if start_frame >= 100_000 {
        start_frame = 0;
    }
    if set_frame < 0.0 && set_frame != -1.0 {
        set_frame = 0.0;
    }
    if set_frame > 100_000.0 {
        set_frame = 0.0;
    }

    if !res {
        return false;
    }
    ghl_info.skel_frame_num = 0;
    // Safety: `res` is true, so `g2_setup_model_pointers` has populated
    // `a_header` from a valid model (G2_API.cpp:1058 dereferences it
    // unconditionally on this path — same faithful no-null-check transcription).
    let num_frames = unsafe { MdxaView::from_block(ghl_info.a_header) }.num_frames();
    crate::bones::g2_set_bone_anim_index(
        &mut ghl_info.blist,
        index,
        start_frame,
        end_frame,
        flags,
        anim_speed,
        current_time,
        set_frame,
        blend_time,
        num_frames,
    )
}

/// Raven `G2API_RemoveBone` — flushes `mSkelFrameNum`, forwards to
/// `G2_Remove_Bone`; `qfalse` on `G2_SetupModelPointers` failure.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:1416-1424`
pub fn g2api_remove_bone(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghl_info: &mut CGhoul2Info,
    bone_name: &str,
) -> bool {
    let _ = g2;
    if !crate::misc::g2_setup_model_pointers(host, ghl_info) {
        return false;
    }
    ghl_info.skel_frame_num = 0;
    let mut blist = mem::take(&mut ghl_info.blist);
    let result = crate::bones::g2_remove_bone(ghl_info, &mut blist, bone_name);
    ghl_info.blist = blist;
    result
}

/// Raven `int G2API_GetBoneIndex` — `-1` on `G2_SetupModelPointers` failure,
/// else `G2_Get_Bone_Index`'s result. A plain `int` return (not `qboolean`),
/// kept as `i32` per §C7 (no bool coercion where Raven's contract is already
/// an integer index/sentinel).
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:2463-2469`
pub fn g2api_get_bone_index(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghl_info: &mut CGhoul2Info,
    bone_name: &str,
) -> i32 {
    let _ = g2;
    if crate::misc::g2_setup_model_pointers(host, ghl_info) {
        crate::bones::g2_get_bone_index(host, ghl_info, bone_name)
    } else {
        -1
    }
}

/// Raven `G2API_DoesBoneExist` — on `G2_SetupModelPointers` success, walks
/// `currentModel->mdxa`'s skeleton bone names (loader model memory) looking
/// for a case-insensitive match; `qfalse` if setup fails, `mdxa` is null, or
/// no bone matches. The `mdxa` skeleton read crosses `EngineHost::model_mdxa`
/// (`G2SV-D5`/`G2SV-D15`) rather than naming the loader's `mdxaHeader_t` type
/// here.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:963-988`
pub fn g2api_does_bone_exist(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghl_info: &mut CGhoul2Info,
    bone_name: &str,
) -> bool {
    let _ = g2;
    if !crate::misc::g2_setup_model_pointers(host, ghl_info) {
        return false;
    }
    let mdxa = ghl_info.a_header;
    if mdxa.is_null() {
        return false;
    }
    // Safety: `mdxa` is a non-null block returned by `g2_setup_model_pointers`
    // (ultimately `EngineHost::model_mdxa`); `num_bones` bounds the walk below.
    let mdxa = unsafe { MdxaView::from_block(mdxa) };
    let num_bones = mdxa.num_bones();
    for i in 0..num_bones {
        if mdxa.skel(i).name_matches(bone_name) {
            return true;
        }
    }
    false
}

/// Raven `void G2API_ListBones(CGhoul2Info *ghlInfo, int frame)` — on
/// `G2_SetupModelPointers` success, dumps the model's skeleton bone list via
/// `G2_List_Model_Bones` (internal `g2_list_model_bones` in `misc.rs`, which
/// threads `host` for its loader model-memory read); no-op if setup fails.
///
/// LIVE via `G_G2_LISTBONES` (`g_public.h:507`, `sv_game.cpp:1316`).
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:1911-1917`
pub fn g2api_list_bones(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghl_info: &mut CGhoul2Info,
    frame: i32,
) {
    let _ = g2;
    if crate::misc::g2_setup_model_pointers(host, ghl_info) {
        crate::misc::g2_list_model_bones(host, &ghl_info.file_name, frame);
    }
}

/// Raven `void G2API_AnimateG2Models(CGhoul2Info_v &ghoul2, float speedVar)`
/// — declared (`G2_local.h:131`) but **never defined** anywhere in
/// `oracle/codemp/` (no `.cpp` body) and never called (every in-tree call
/// site — `sv_game.cpp:1554`, `client/cl_cgame.cpp:1589` — targets the other
/// `G2API_AnimateG2Models(CGhoul2Info_v&, int, CRagDollUpdateParams*)`
/// overload, ported separately as `g2api_animate_g2_models_rag` in
/// `api_ragdoll.rs`). Ported per the doc's explicit roster listing
/// ("`AnimateG2Models`", `api_bones.rs` row) with the header's signature,
/// matching the sibling `g2api_detach_ent` precedent in `api_bolts.rs` (a
/// declared-only Raven prototype with no definition to classify out-param
/// behavior against); no `host` thread since no body exists to show one is
/// needed.
///
/// Source: `oracle/codemp/ghoul2/G2_local.h:131` (no `.cpp` definition found)
pub fn g2api_animate_g2_models(g2: &mut Ghoul2System, ghoul2: &mut CGhoul2Info_v, speed_var: f32) {
    // No oracle .cpp body exists anywhere in codemp/ (see doc comment above) —
    // a compiled no-op is the faithful transcription of "no definition".
    let _ = (g2, ghoul2, speed_var);
}

#[cfg(test)]
mod tests {
    // The `mdxaHeader_t`/`mdxaSkel_t` byte-layout reads this file used to check
    // locally now live in `mp_host_interface::mdx::mdxa`'s own tests.

    #[test]
    fn set_bone_anim_clamps_out_of_range_frames() {
        // Mirrors the clamp block shared by g2api_set_bone_anim/
        // g2api_set_bone_anim_index (G2_API.cpp:1069-1093/1030-1054): this
        // doesn't call either wrapper (both forward into `bones.rs`, still
        // `todo!()`), just re-checks the clamp arithmetic in isolation.
        let clamp = |mut end_frame: i32, mut start_frame: i32, mut set_frame: f32| {
            if end_frame <= 0 {
                end_frame = 1;
            }
            if end_frame >= 100_000 {
                end_frame = 1;
            }
            if start_frame < 0 {
                start_frame = 0;
            }
            if start_frame >= 100_000 {
                start_frame = 0;
            }
            if set_frame < 0.0 && set_frame != -1.0 {
                set_frame = 0.0;
            }
            if set_frame > 100_000.0 {
                set_frame = 0.0;
            }
            (end_frame, start_frame, set_frame)
        };

        assert_eq!(clamp(0, -1, -5.0), (1, 0, 0.0));
        assert_eq!(clamp(200_000, 200_000, 200_000.0), (1, 0, 0.0));
        // -1.0 is the sentinel "keep current frame" value and must survive.
        assert_eq!(clamp(10, 0, -1.0), (10, 0, -1.0));
        assert_eq!(clamp(10, 5, 3.0), (10, 5, 3.0));
    }
}
