//! `G2_Bones internal` — the non-ragdoll bone-list mutators/queries
//! (`G2_bones.cpp`'s "internal bone calls" surface, `G2_local.h:27-62`) that
//! `api_bones.rs`'s `G2API_*` wrappers forward into.
//!
//! Per `docs/subsystems/ghoul2-server.md` roster (`bones.rs`, class "G2_Bones
//! internal"): "`G2_Set/Get/Stop/Pause_Bone_Anim/Angles(+Matrix,+Index)`,
//! `G2_Find_Bone_In_List`, `G2_Init_Bone_List`,
//! `G2_RemoveRedundantBoneOverrides`, `G2_Animate_Bone_List` (non-ragdoll bone
//! logic)". Expanded against `G2_local.h:27-62` ("internal bone calls -
//! G2_Bones.cpp", the header section that is this class's own boundary) plus
//! the file-scope private helpers (`G2_Find_Bone`, `G2_Add_Bone`,
//! `G2_Remove_Bone_Index`, `G2_Stop_Bone_Index`, `G2_Generate_Matrix`) those
//! declared functions share — 21 declared + 5 private = 26 functions, ported
//! in full below (porting-rules §F21 one-class-per-file; "private helpers
//! included" per the assignment).
//!
//! **Two findings reported upstream (see the porting task's `problems`
//! output, kept out of the doc's own text per house rules (never edit the
//! frozen doc from a porter file):**
//! 1. The doc's `G2_Animate_Bone_List (non-ragdoll bone logic)` label matches
//!    the **dead** 3-arg overload (`G2_bones.cpp:1065-1136`, zero callers
//!    anywhere in `oracle/codemp/`, not declared in `G2_local.h`) rather than
//!    the **live**, header-declared, `G2_API.cpp:1450`-called 4-arg overload
//!    (`:4498-4527`, taking `CRagDollUpdateParams *params` and dispatching
//!    into `G2_DoIK`/`G2_RagDoll`). Only the live 4-arg overload is ported here
//!    (`g2_animate_bone_list` below); the dead 3-arg one gets this §20
//!    zero-caller note, not a stub.
//! 2. `G2_Stop_Bone_Anim`/`G2_Stop_Bone_Angles`/`G2_IsPaused`/`G2_Get_Bone_Index`
//!    (and `G2_Set_Bone_Angles_Matrix`'s empty-`fileName` fallback) resolve a
//!    model **by filename** — `R_GetModelByHandle(RE_RegisterModel(fileName))`
//!    (`G2_bones.cpp:971-990,1009-1025,1045-1061,4902-4908`) — a
//!    register-by-name + handle-to-pointer service the frozen 15-method
//!    `EngineHost` (`## Seam definition`) does not expose (`model_mdxm`/
//!    `model_mdxa` both take an already-held `qhandle_t`, not a filename, and
//!    return the raw parsed block, not a `model_t*`). This is a real gap in
//!    the doc's "`G2SV-Q11` SETTLED, no gap remains" claim, not a per-function
//!    judgment call; each affected function below threads `host` and cites the
//!    gap at its own site rather than inventing an unfrozen method. Where the
//!    caller already hands over a resolved pointer or a `qhandle_t` instead of
//!    a bare filename (`g2_get_bone_index`'s `CGhoul2Info` param,
//!    `g2_set_bone_angles_matrix`'s empty-`fileName` `modelList[modelIndex]`
//!    arm), the gap does not apply and the body resolves the model for real.
//!
//! **Further findings reported upstream while filling this file's bodies:**
//! 4. `G2_TimingModel` (`oracle/codemp/renderer/tr_ghoul2.cpp:1167-1407`) is
//!    called from two sites — `G2_TransformBone` (`render/bone_transform.rs`,
//!    `:1596`) and `G2_bones.cpp:885` (this file's `g2_get_bone_anim_index`).
//!    Its single canonical home is `render/bone_transform.rs`'s `pub(crate)`
//!    [`g2_timing_model`](crate::render::bone_transform::g2_timing_model),
//!    imported here (the earlier local duplicate is retired).
//! 5. That canonical takes `&mut boneInfo_t` and performs Raven's one write
//!    (`bone.flags &= ~(BONE_ANIM_TOTAL)`, the "not override-loop, not
//!    override-freeze, ran off the end" arm, `tr_ghoul2.cpp:1310`). But
//!    `g2_get_bone_anim_index`'s frozen skeleton signature takes
//!    `blist: &[boneInfo_t]` (read-only, pinned — porting-rules §F: signatures
//!    are LAW), so it cannot and must not observe that write. It passes the
//!    canonical a scratch clone of the bone; the write lands on the throwaway
//!    copy and the real `blist` entry is untouched — reproducing this call
//!    chain's original no-observed-write behavior exactly.

use mp_host_interface::EngineHost;
use mp_qshared::shared::q_math::Create_Matrix;
use mp_qshared::shared::{mdxaBone_t, qhandle_t, vec3_t, Eorientations};

use crate::ghoul2_system::Ghoul2System;
use mp_host_interface::mdx::mdxa::MdxaView;
use crate::ragdoll_update_params::RagDollUpdateParams;
use crate::render::bone_transform::g2_timing_model;
use crate::shared::bone_info_t::boneInfo_t;
use crate::shared::cghoul2_info::CGhoul2Info;
use crate::shared::cghoul2_info_v::CGhoul2Info_v;

// ---------------------------------------------------------------------------
// `boneInfo_t::flags`/`RagFlags` bit constants (`G2.h:8-26`, `G2_bones.cpp:1200`).
// Duplicated locally (this crate has no shared flags-constants module yet;
// `api_models.rs`'s `GHOUL2_NEWORIGIN` and `ghoul2_system.rs`/`info_array.rs`'s
// `MAX_G2_MODELS`/`G2_INDEX_MASK` are already duplicated the same way between
// files, per their own doc comments) rather than invented as a new cross-file
// dependency.
// ---------------------------------------------------------------------------

/// Source: `oracle/codemp/ghoul2/G2.h:8`
const BONE_ANGLES_PREMULT: i32 = 0x0001;
/// Source: `oracle/codemp/ghoul2/G2.h:9`
const BONE_ANGLES_POSTMULT: i32 = 0x0002;
/// Source: `oracle/codemp/ghoul2/G2.h:10`
const BONE_ANGLES_REPLACE: i32 = 0x0004;
/// Source: `oracle/codemp/ghoul2/G2.h:17`
const BONE_ANGLES_RAGDOLL: i32 = 0x2000;
/// Source: `oracle/codemp/ghoul2/G2.h:21`
const BONE_ANGLES_TOTAL: i32 = BONE_ANGLES_PREMULT | BONE_ANGLES_POSTMULT | BONE_ANGLES_REPLACE;
/// Source: `oracle/codemp/ghoul2/G2.h:22`
const BONE_ANIM_OVERRIDE: i32 = 0x0008;
/// Source: `oracle/codemp/ghoul2/G2.h:23`
const BONE_ANIM_OVERRIDE_LOOP: i32 = 0x0010;
/// Source: `oracle/codemp/ghoul2/G2.h:24`
const BONE_ANIM_OVERRIDE_FREEZE: i32 = 0x0040 + BONE_ANIM_OVERRIDE;
/// Source: `oracle/codemp/ghoul2/G2.h:25`
const BONE_ANIM_BLEND: i32 = 0x0080;
/// Source: `oracle/codemp/ghoul2/G2.h:26`
const BONE_ANIM_TOTAL: i32 =
    BONE_ANIM_OVERRIDE | BONE_ANIM_OVERRIDE_LOOP | BONE_ANIM_OVERRIDE_FREEZE | BONE_ANIM_BLEND;
/// Source: `oracle/codemp/ghoul2/G2.h:14`
const BONE_NEED_TRANSFORM: i32 = 0x8000;
/// Raven `#define RAG_PCJ_IK_CONTROLLED (0x08000)` — a `boneInfo_t::RagFlags`
/// bit, distinct from (if numerically identical to) `BONE_NEED_TRANSFORM`
/// above, which lives on the sibling `flags` field.
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:1200`
const RAG_PCJ_IK_CONTROLLED: i32 = 0x08000;

// ---------------------------------------------------------------------------
// Private helpers (file-scope in the oracle .cpp, not declared in
// G2_local.h; used only from the functions in this file).
// ---------------------------------------------------------------------------

/// Raven `int G2_Find_Bone(const model_t *mod, boneInfo_v &blist, const char
/// *boneName)` — linear-scans `blist` for an override entry whose bone number
/// resolves (via `mod`'s skeleton, `.gla`) to `boneName`; `-1` if not found.
/// `mod` here is the already-resolved anim-model pointer (`CGhoul2Info::
/// anim_model`'s type, `*const c_void`, `G2SV-D5`) — never `model_t`/
/// `mdxaHeader_t` named as a Rust type.
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:38-69`
pub fn g2_find_bone(anim_model: Option<MdxaView<'static>>, blist: &[boneInfo_t], bone_name: &str) -> i32 {
    for (i, bone) in blist.iter().enumerate() {
        if bone.boneNumber == -1 {
            continue;
        }
        // An active bone slot's `boneNumber` indexes `anim_model`'s skeleton;
        // callers hold `anim_model` non-`None` whenever `blist` carries an
        // active bone, matching Raven's unchecked `mod->mdxa` deref.
        if anim_model
            .expect("G2_Find_Bone: null anim model with active bone")
            .skel(bone.boneNumber)
            .name_matches(bone_name)
        {
            return i as i32;
        }
    }
    -1
}

/// Raven `int G2_Add_Bone (const model_t *mod, boneInfo_v &blist, const char
/// *boneName)` — finds `boneName` in `mod`'s skeleton, then either reuses a
/// free (`boneNumber == -1`) slot in `blist` or `push_back`s a new one;
/// `-1` if `boneName` has no skeleton match.
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:71-141`
pub fn g2_add_bone(anim_model: Option<MdxaView<'static>>, blist: &mut Vec<boneInfo_t>, bone_name: &str) -> i32 {
    // Caller holds a valid, non-`None` anim model whenever it expects a bone to
    // actually be added (matches Raven's unchecked `mod->mdxa->numBones` deref).
    let mdxa = anim_model.expect("G2_Add_Bone: null anim model");
    let num_bones = mdxa.num_bones();
    let mut x = 0i32;
    while x < num_bones {
        if mdxa.skel(x).name_matches(bone_name) {
            break;
        }
        x += 1;
    }
    if x == num_bones {
        // Raven's "WARNING: Failed to add bone" print is `_DEBUG`/
        // `_RAG_PRINT_TEST`-only, neither defined in the WinDed NDEBUG build
        // (misc.rs's established treatment of the same class of debug print).
        return -1;
    }

    for (i, bone) in blist.iter_mut().enumerate() {
        if bone.boneNumber != -1 {
            if mdxa.skel(bone.boneNumber).name_matches(bone_name) {
                return i as i32;
            }
        } else {
            bone.boneNumber = x;
            bone.flags = 0;
            return i as i32;
        }
    }

    // Raven: `memset(&tempBone, 0, sizeof(tempBone));` then set boneNumber/flags.
    let mut temp_bone: boneInfo_t = unsafe { core::mem::zeroed() };
    temp_bone.boneNumber = x;
    temp_bone.flags = 0;
    blist.push(temp_bone);
    (blist.len() - 1) as i32
}

/// Raven `qboolean G2_Remove_Bone_Index ( boneInfo_v &blist, int index)` —
/// `qtrue` (no-op) on a ragdoll bone; else, if the slot's `flags` are clear,
/// marks it unused and shrinks `blist` off any trailing run of unused slots.
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:144-193`
pub fn g2_remove_bone_index(blist: &mut Vec<boneInfo_t>, index: i32) -> bool {
    if index != -1 && blist[index as usize].flags & BONE_ANGLES_RAGDOLL != 0 {
        return true; // don't accept any calls on ragdoll bones
    }

    if index != -1 && blist[index as usize].flags == 0 {
        blist[index as usize].boneNumber = -1;

        let mut new_size = blist.len();
        let mut i = blist.len() as isize - 1;
        while i > -1 {
            if blist[i as usize].boneNumber == -1 {
                new_size = i as usize;
            } else {
                break;
            }
            i -= 1;
        }
        if new_size != blist.len() {
            blist.truncate(new_size);
        }
        return true;
    }

    false
}

/// Raven `qboolean G2_Stop_Bone_Index( boneInfo_v &blist, int index, int
/// flags)` — clears `flags` on the slot then forwards to
/// `g2_remove_bone_index`. Its sole Raven caller is inside the dead 3-arg
/// `G2_Animate_Bone_List` overload (`G2_bones.cpp:1135`, module doc finding
/// 1's zero-caller sibling), so this helper is itself currently unreached in
/// this build — ported anyway per the assignment's "private helpers
/// included".
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:211-225`
pub fn g2_stop_bone_index(blist: &mut Vec<boneInfo_t>, index: i32, flags: i32) -> bool {
    if index != -1 {
        blist[index as usize].flags &= !flags;
        return g2_remove_bone_index(blist, index);
    }
    // Raven: `assert(0); return qfalse;` — unreachable in practice (every
    // live call site passes an already-found/added index).
    false
}

/// Raven `void G2_Generate_Matrix(const model_t *mod, boneInfo_v &blist, int
/// index, const float *angles, int flags, const Eorientations up, left,
/// forward)` — builds `blist[index].matrix`/`newMatrix` from `angles` (+ the
/// skeleton's base pose read off `mod` when `flags` has
/// `BONE_ANGLES_PREMULT`/`POSTMULT`; `mod` may be null otherwise, matching
/// Raven's `G2_Generate_Matrix(NULL, ...)` call at `:471`).
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:227-419`
#[allow(clippy::too_many_arguments)]
pub fn g2_generate_matrix(
    anim_model: Option<MdxaView<'static>>,
    blist: &mut Vec<boneInfo_t>,
    index: i32,
    angles: vec3_t,
    flags: i32,
    up: Eorientations,
    left: Eorientations,
    forward: Eorientations,
) {
    let idx = index as usize;

    let bone_override = if flags & (BONE_ANGLES_PREMULT | BONE_ANGLES_POSTMULT) != 0 {
        // Build a matrix out of the fed angles, swapping y/z per Raven's
        // "wacky Quake setup" comment. Raven's `newAngles` here is an
        // uninitialized stack local when `up`/`left`/`forward` is `ORIGIN`
        // (no `default:` arm in any of the three switches) — genuine Raven
        // UB (§19); this picks the defined fallback of leaving that
        // component at the zero this local is pre-seeded with.
        let mut new_angles: vec3_t = [0.0; 3];
        new_angles[1] = match up {
            Eorientations::NEGATIVE_X => angles[2] + 180.0,
            Eorientations::POSITIVE_X => angles[2],
            Eorientations::NEGATIVE_Y => angles[0],
            Eorientations::POSITIVE_Y => angles[0],
            Eorientations::NEGATIVE_Z => angles[1] + 180.0,
            Eorientations::POSITIVE_Z => angles[1],
            Eorientations::ORIGIN => new_angles[1],
        };
        new_angles[0] = match left {
            Eorientations::NEGATIVE_X => angles[2],
            Eorientations::POSITIVE_X => angles[2] + 180.0,
            Eorientations::NEGATIVE_Y => angles[0],
            Eorientations::POSITIVE_Y => angles[0] + 180.0,
            Eorientations::NEGATIVE_Z => angles[1],
            Eorientations::POSITIVE_Z => angles[1],
            Eorientations::ORIGIN => new_angles[0],
        };
        new_angles[2] = match forward {
            Eorientations::NEGATIVE_X => angles[2],
            Eorientations::POSITIVE_X => angles[2],
            Eorientations::NEGATIVE_Y => angles[0],
            Eorientations::POSITIVE_Y => angles[0] + 180.0,
            Eorientations::NEGATIVE_Z => angles[1],
            Eorientations::POSITIVE_Z => angles[1] + 180.0,
            Eorientations::ORIGIN => new_angles[2],
        };

        let mut bone_override = mdxaBone_t {
            matrix: [[0.0; 4]; 3],
        };
        Create_Matrix(new_angles, &mut bone_override);

        let bone_number = blist[idx].boneNumber;
        // PREMULT/POSTMULT callers always pass a resolved `anim_model` (matches
        // Raven's unchecked `mod->mdxa` dereference; the `*_Index` sibling
        // rejects these flags before ever reaching here with a null model, per
        // its own doc comment below).
        let skel = anim_model.expect("G2_Generate_Matrix: null anim model under PRE/POSTMULT").skel(bone_number);
        let base_pose_mat = skel.base_pose_mat();
        let base_pose_mat_inv = skel.base_pose_mat_inv();

        let mut temp1 = mdxaBone_t {
            matrix: [[0.0; 4]; 3],
        };
        crate::render::bone_transform::multiply_3x4_matrix(
            &mut temp1,
            &bone_override,
            &base_pose_mat_inv,
        );
        crate::render::bone_transform::multiply_3x4_matrix(
            &mut bone_override,
            &base_pose_mat,
            &temp1,
        );
        bone_override
    } else {
        let mut new_angles = angles;
        // "why I should need do this Fuck alone knows. But I do."
        if matches!(left, Eorientations::POSITIVE_Y) {
            new_angles[0] += 180.0;
        }
        let mut temp1 = mdxaBone_t {
            matrix: [[0.0; 4]; 3],
        };
        Create_Matrix(new_angles, &mut temp1);

        // Raven explicitly zeroes all 12 `permutation.matrix` cells before
        // the switches below; the zero-initialized literal here already
        // matches that (so an `ORIGIN` arm leaving a cell untouched is
        // well-defined, unlike the PREMULT/POSTMULT branch's `newAngles`).
        let mut permutation = mdxaBone_t {
            matrix: [[0.0; 4]; 3],
        };
        match forward {
            Eorientations::NEGATIVE_X => permutation.matrix[0][0] = -1.0,
            Eorientations::POSITIVE_X => permutation.matrix[0][0] = 1.0,
            Eorientations::NEGATIVE_Y => permutation.matrix[1][0] = -1.0,
            Eorientations::POSITIVE_Y => permutation.matrix[1][0] = 1.0,
            Eorientations::NEGATIVE_Z => permutation.matrix[2][0] = -1.0,
            Eorientations::POSITIVE_Z => permutation.matrix[2][0] = 1.0,
            Eorientations::ORIGIN => {}
        }
        match left {
            Eorientations::NEGATIVE_X => permutation.matrix[0][1] = -1.0,
            Eorientations::POSITIVE_X => permutation.matrix[0][1] = 1.0,
            Eorientations::NEGATIVE_Y => permutation.matrix[1][1] = -1.0,
            Eorientations::POSITIVE_Y => permutation.matrix[1][1] = 1.0,
            Eorientations::NEGATIVE_Z => permutation.matrix[2][1] = -1.0,
            Eorientations::POSITIVE_Z => permutation.matrix[2][1] = 1.0,
            Eorientations::ORIGIN => {}
        }
        match up {
            Eorientations::NEGATIVE_X => permutation.matrix[0][2] = -1.0,
            Eorientations::POSITIVE_X => permutation.matrix[0][2] = 1.0,
            Eorientations::NEGATIVE_Y => permutation.matrix[1][2] = -1.0,
            Eorientations::POSITIVE_Y => permutation.matrix[1][2] = 1.0,
            Eorientations::NEGATIVE_Z => permutation.matrix[2][2] = -1.0,
            Eorientations::POSITIVE_Z => permutation.matrix[2][2] = 1.0,
            Eorientations::ORIGIN => {}
        }

        let mut bone_override = mdxaBone_t {
            matrix: [[0.0; 4]; 3],
        };
        crate::render::bone_transform::multiply_3x4_matrix(
            &mut bone_override,
            &temp1,
            &permutation,
        );
        bone_override
    };

    // Raven: `memcpy(&blist[index].newMatrix, &blist[index].matrix, ...)` —
    // `boneOverride` already *was* `&blist[index].matrix` in the oracle, so
    // both fields end up holding the same final value.
    blist[idx].matrix = bone_override;
    blist[idx].newMatrix = bone_override;
}

// ---------------------------------------------------------------------------
// Header-declared internal bone calls (G2_local.h:27-62), in that order.
// ---------------------------------------------------------------------------

/// Raven `qboolean G2_Set_Bone_Angles(CGhoul2Info *ghlInfo, boneInfo_v &blist,
/// const char *boneName, const float *angles, const int flags, const
/// Eorientations up, left, forward, qhandle_t *modelList, const int
/// modelIndex, const int blendTime, const int currentTime)` — finds or adds
/// `boneName` (via `ghl_info.anim_model`), rejects ragdoll bones (`qtrue`
/// no-op), else sets flags/blend timing and forwards to `g2_generate_matrix`.
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:476-533`
#[allow(clippy::too_many_arguments)]
pub fn g2_set_bone_angles(
    ghl_info: &CGhoul2Info,
    blist: &mut Vec<boneInfo_t>,
    bone_name: &str,
    angles: vec3_t,
    flags: i32,
    up: Eorientations,
    left: Eorientations,
    forward: Eorientations,
    model_list: &[qhandle_t],
    model_index: i32,
    blend_time: i32,
    current_time: i32,
) -> bool {
    // Raven never reads `modelList`/`modelIndex` in this function's body.
    let _ = (model_list, model_index);
    let mod_a = ghl_info.anim_model;

    let mut index = g2_find_bone(mod_a, blist, bone_name);
    if index != -1 {
        if blist[index as usize].flags & BONE_ANGLES_RAGDOLL != 0 {
            return true;
        }
        blist[index as usize].flags &= !BONE_ANGLES_TOTAL;
        blist[index as usize].flags |= flags;
        blist[index as usize].boneBlendStart = current_time;
        blist[index as usize].boneBlendTime = blend_time;
        g2_generate_matrix(mod_a, blist, index, angles, flags, up, left, forward);
        return true;
    }

    index = g2_add_bone(mod_a, blist, bone_name);
    if index != -1 {
        blist[index as usize].flags &= !BONE_ANGLES_TOTAL;
        blist[index as usize].flags |= flags;
        blist[index as usize].boneBlendStart = current_time;
        blist[index as usize].boneBlendTime = blend_time;
        g2_generate_matrix(mod_a, blist, index, angles, flags, up, left, forward);
        return true;
    }
    // Raven's own comment: "we don't need an assert here too. There's already
    // a warning in G2_Add_Bone if it fails."
    false
}

/// Raven `qboolean G2_Remove_Bone(CGhoul2Info *ghlInfo, boneInfo_v &blist,
/// const char *boneName)` — finds `boneName` via `ghl_info.anim_model`, then
/// forwards to `g2_remove_bone_index`.
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:420-429`
pub fn g2_remove_bone(
    ghl_info: &CGhoul2Info,
    blist: &mut Vec<boneInfo_t>,
    bone_name: &str,
) -> bool {
    debug_assert!(ghl_info.anim_model.is_some());
    let index = g2_find_bone(ghl_info.anim_model, blist, bone_name);
    g2_remove_bone_index(blist, index)
}

/// Raven `qboolean G2_Set_Bone_Anim(CGhoul2Info *ghlInfo, boneInfo_v &blist,
/// const char *boneName, const int startFrame, endFrame, flags, const float
/// animSpeed, const int currentTime, const float setFrame, const int
/// blendTime)` — finds or adds `boneName`, rejects ragdoll bones, else
/// forwards to `g2_set_bone_anim_index` with `ghl_info.a_header`'s
/// `numFrames`.
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:814-847`
#[allow(clippy::too_many_arguments)]
pub fn g2_set_bone_anim(
    ghl_info: &CGhoul2Info,
    blist: &mut Vec<boneInfo_t>,
    bone_name: &str,
    start_frame: i32,
    end_frame: i32,
    flags: i32,
    anim_speed: f32,
    current_time: i32,
    set_frame: f32,
    blend_time: i32,
) -> bool {
    let mod_a = ghl_info.anim_model;
    let mut index = g2_find_bone(mod_a, blist, bone_name);
    if index == -1 {
        index = g2_add_bone(mod_a, blist, bone_name);
    }
    if index != -1 && blist[index as usize].flags & BONE_ANGLES_RAGDOLL != 0 {
        return true;
    }
    if index != -1 {
        // `ghl_info.a_header` is a non-`None` `EngineHost::model_mdxa` block
        // whenever this is reached with a valid instance (matches Raven's
        // unchecked `ghlInfo->aHeader->numFrames` dereference).
        let num_frames = ghl_info.a_header.expect("G2_Set_Bone_Anim: null aHeader").num_frames();
        return g2_set_bone_anim_index(
            blist,
            index,
            start_frame,
            end_frame,
            flags,
            anim_speed,
            current_time,
            set_frame,
            blend_time,
            num_frames,
        );
    }
    false
}

/// Raven `qboolean G2_Get_Bone_Anim_Range(CGhoul2Info *ghlInfo, boneInfo_v
/// &blist, const char *boneName, int *startFrame, int *endFrame)` — finds
/// `boneName` via `ghl_info.anim_model`; if it is an animating bone, writes
/// its start/end frame and returns `qtrue`, else leaves the out-params
/// untouched and returns `qfalse`.
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:849-866`
pub fn g2_get_bone_anim_range(
    ghl_info: &CGhoul2Info,
    blist: &[boneInfo_t],
    bone_name: &str,
    start_frame: &mut i32,
    end_frame: &mut i32,
) -> bool {
    let index = g2_find_bone(ghl_info.anim_model, blist, bone_name);
    if index != -1 {
        let bone = &blist[index as usize];
        if bone.flags & (BONE_ANIM_OVERRIDE_LOOP | BONE_ANIM_OVERRIDE) != 0 {
            *start_frame = bone.startFrame;
            *end_frame = bone.endFrame;
            return true;
        }
    }
    false
}

/// Raven `qboolean G2_Pause_Bone_Anim(CGhoul2Info *ghlInfo, boneInfo_v &blist,
/// const char *boneName, const int currentTime)` — finds `boneName` via
/// `ghl_info.anim_model`; toggles `pauseTime` (un-pausing re-derives the
/// current frame via `g2_get_bone_anim`/re-sets via `g2_set_bone_anim`).
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:936-969`
pub fn g2_pause_bone_anim(
    ghl_info: &CGhoul2Info,
    blist: &mut Vec<boneInfo_t>,
    bone_name: &str,
    current_time: i32,
) -> bool {
    let index = g2_find_bone(ghl_info.anim_model, blist, bone_name);
    if index != -1 {
        let pause_time = blist[index as usize].pauseTime;
        if pause_time != 0 {
            let mut current_frame = 0.0f32;
            let mut start_frame = 0i32;
            let mut end_frame = 0i32;
            let mut flags = 0i32;
            let mut anim_speed = 0.0f32;
            g2_get_bone_anim(
                ghl_info,
                blist,
                bone_name,
                pause_time,
                &mut current_frame,
                &mut start_frame,
                &mut end_frame,
                &mut flags,
                &mut anim_speed,
                &[],
                0,
            );
            g2_set_bone_anim(
                ghl_info,
                blist,
                bone_name,
                start_frame,
                end_frame,
                flags,
                anim_speed,
                current_time,
                current_frame,
                0,
            );
            blist[index as usize].pauseTime = 0;
        } else {
            blist[index as usize].pauseTime = current_time;
        }
        return true;
    }
    // Raven: `assert(0); return qfalse;`
    false
}

/// Raven `qboolean G2_IsPaused(const char *fileName, boneInfo_v &blist, const
/// char *boneName)` — resolves the anim model **by filename**
/// (`R_GetModelByHandle(RE_RegisterModel(fileName))` then
/// `->mdxm->animIndex`, `:973-974`) and reports whether the found bone's
/// `pauseTime` is set. `host` is threaded for this resolution, but the
/// register-by-name + handle-to-pointer chain has **no** corresponding
/// `EngineHost` method (reported upstream — see the module doc's finding 2);
/// this returns the same "not found" result a resolvable-but-non-matching
/// lookup would give.
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:971-990`
pub fn g2_is_paused(
    host: &mut impl EngineHost,
    file_name: &str,
    blist: &[boneInfo_t],
    bone_name: &str,
) -> bool {
    let _ = (host, file_name, blist, bone_name);
    false
}

/// Raven `qboolean G2_Stop_Bone_Anim(const char *fileName, boneInfo_v &blist,
/// const char *boneName)` — same by-filename model resolution as
/// `g2_is_paused` (`:1011-1012`, same unserved gap), then forwards to
/// `g2_remove_bone_index` after clearing the anim flags.
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:1009-1025`
pub fn g2_stop_bone_anim(
    host: &mut impl EngineHost,
    file_name: &str,
    blist: &mut Vec<boneInfo_t>,
    bone_name: &str,
) -> bool {
    let _ = (host, file_name, blist, bone_name);
    false
}

/// Raven `qboolean G2_Stop_Bone_Angles(const char *fileName, boneInfo_v
/// &blist, const char *boneName)` — same by-filename model resolution as
/// `g2_is_paused` (`:1047-1048`, same unserved gap), then forwards to
/// `g2_remove_bone_index` after clearing the angle-override flags.
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:1045-1061`
pub fn g2_stop_bone_angles(
    host: &mut impl EngineHost,
    file_name: &str,
    blist: &mut Vec<boneInfo_t>,
    bone_name: &str,
) -> bool {
    let _ = (host, file_name, blist, bone_name);
    false
}

/// Raven `qboolean G2_Get_Bone_Anim(CGhoul2Info *ghlInfo, boneInfo_v &blist,
/// const char *boneName, const int currentTime, float *currentFrame, int
/// *startFrame, *endFrame, *flags, float *retAnimSpeed, qhandle_t *modelList,
/// int modelIndex)` — finds or adds `boneName`, then forwards to
/// `g2_get_bone_anim_index`. Out-params kept as `&mut` 1:1 (this is an
/// internal helper, not a `G2API_*` surface function — the `G2SV-D1`
/// write-through discriminator is scoped to the `G2API_*` surface, not this
/// internal set).
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:904-933`
#[allow(clippy::too_many_arguments)]
pub fn g2_get_bone_anim(
    ghl_info: &CGhoul2Info,
    blist: &mut Vec<boneInfo_t>,
    bone_name: &str,
    current_time: i32,
    current_frame: &mut f32,
    start_frame: &mut i32,
    end_frame: &mut i32,
    flags: &mut i32,
    ret_anim_speed: &mut f32,
    model_list: &[qhandle_t],
    model_index: i32,
) -> bool {
    // Raven never reads `modelIndex` in this function's body (`modelList` is
    // forwarded but `g2_get_bone_anim_index` itself never reads it either).
    let _ = model_index;
    let mod_a = ghl_info.anim_model;
    let mut index = g2_find_bone(mod_a, blist, bone_name);
    if index == -1 {
        index = g2_add_bone(mod_a, blist, bone_name);
        if index == -1 {
            return false;
        }
    }
    debug_assert!(ghl_info.a_header.is_some());
    // See the debug_assert above; matches Raven's unchecked
    // `ghlInfo->aHeader->numFrames` dereference.
    let num_frames = ghl_info.a_header.expect("G2_Get_Bone_Anim: null aHeader").num_frames();
    g2_get_bone_anim_index(
        blist.as_slice(),
        index,
        current_time,
        current_frame,
        start_frame,
        end_frame,
        flags,
        ret_anim_speed,
        model_list,
        num_frames,
    )
}

/// Raven `void G2_Animate_Bone_List(CGhoul2Info_v &ghoul2, const int
/// currentTime, const int index, CRagDollUpdateParams *params)` — the
/// **live** overload (`G2_local.h:42`, called from `G2API_AnimateG2Models`,
/// `G2_API.cpp:1450`): scans `ghoul2[index].mBlist` for any ragdoll/IK
/// override bone, then — only for `index == 0` with `params` non-null —
/// dispatches to `G2_DoIK` (`ragdoll.rs`) or `G2_RagDoll` (`ragdoll.rs`). The
/// comment at `:4496` ("cut out the entire non-ragdoll section of this..")
/// confirms the non-ragdoll bone-anim-expiry walk the doc's summary describes
/// now lives only in the dead 3-arg sibling overload (module doc finding 1);
/// this live overload does no non-ragdoll work itself. `params` is Raven's
/// nullable `CRagDollUpdateParams *`, so `Option<&mut RagDollUpdateParams>`
/// (`G2SV-D8` enum shape) per porting-rules §C7.
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:4497-4527`
pub fn g2_animate_bone_list(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghoul2: &mut CGhoul2Info_v,
    current_time: i32,
    index: i32,
    params: Option<&mut RagDollUpdateParams>,
) {
    let mut any_ik = false;
    {
        let info = ghoul2.get(g2, index);
        // Raven's `anyRagDoll` only ever gates the early-exit `break` below
        // (matching its own C++ source, which never reads it afterward
        // either) — the compiler flags the final loop iteration's write as
        // dead since nothing reads it once the loop ends.
        #[allow(unused_assignments)]
        let mut any_ragdoll = false;
        for bone in &info.blist {
            if bone.boneNumber != -1 && bone.flags & BONE_ANGLES_RAGDOLL != 0 {
                if bone.RagFlags & RAG_PCJ_IK_CONTROLLED != 0 {
                    any_ik = true;
                }
                any_ragdoll = true;
                if any_ik && any_ragdoll {
                    break;
                }
            }
        }
    }

    if index == 0 {
        if let Some(params) = params {
            if any_ik {
                crate::ragdoll::g2_do_ik(g2, host, ghoul2, 0, Some(params));
            } else {
                crate::ragdoll::g2_rag_doll(g2, host, ghoul2, 0, Some(params), current_time);
            }
        }
    }
}

/// Raven `void G2_Init_Bone_List(boneInfo_v &blist)` — `blist.clear()`.
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:4879-4882`
pub fn g2_init_bone_list(blist: &mut Vec<boneInfo_t>) {
    blist.clear();
}

/// Raven `int G2_Find_Bone_In_List(boneInfo_v &blist, const int boneNum)` —
/// linear scan for the override entry whose `boneNumber == boneNum`; `-1` if
/// none.
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:196-209`
pub fn g2_find_bone_in_list(blist: &[boneInfo_t], bone_num: i32) -> i32 {
    for (i, bone) in blist.iter().enumerate() {
        if bone.boneNumber == bone_num {
            return i as i32;
        }
    }
    -1
}

/// Raven `void G2_RemoveRedundantBoneOverrides(boneInfo_v &blist, int
/// *activeBones)` — clears and removes any override entry whose bone number
/// is not marked active in `activeBones` (indexed by bone number).
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:4884-4900`
pub fn g2_remove_redundant_bone_overrides(blist: &mut Vec<boneInfo_t>, active_bones: &[i32]) {
    // `blist.len()` re-checked each iteration (not cached) because
    // `g2_remove_bone_index` can shrink `blist` mid-loop, matching Raven's
    // own re-evaluated `i<blist.size()` for-loop condition.
    let mut i = 0usize;
    while i < blist.len() {
        if blist[i].boneNumber != -1 && active_bones[blist[i].boneNumber as usize] == 0 {
            blist[i].flags = 0;
            g2_remove_bone_index(blist, i as i32);
        }
        i += 1;
    }
}

/// Raven `qboolean G2_Set_Bone_Angles_Matrix(const char *fileName, boneInfo_v
/// &blist, const char *boneName, const mdxaBone_t &matrix, const int flags,
/// qhandle_t *modelList, const int modelIndex, const int blendTime, const int
/// currentTime)` — finds or adds `boneName` (model resolved from
/// `modelList[modelIndex]` when `fileName` is empty — a `qhandle_t` path
/// `EngineHost::model_mdxa` can serve — else via `RE_RegisterModel(fileName)`,
/// `:577`, the same unserved by-filename gap as `g2_is_paused`), then copies
/// `matrix` into the slot's `matrix`/`newMatrix`.
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:566-620`
#[allow(clippy::too_many_arguments)]
pub fn g2_set_bone_angles_matrix(
    host: &mut impl EngineHost,
    file_name: &str,
    blist: &mut Vec<boneInfo_t>,
    bone_name: &str,
    matrix: &mdxaBone_t,
    flags: i32,
    model_list: &[qhandle_t],
    model_index: i32,
    blend_time: i32,
    current_time: i32,
) -> bool {
    // Raven never reads `blendTime`/`currentTime` in this function's body.
    let _ = (blend_time, current_time);

    let mod_a = if file_name.is_empty() {
        // Divergence (§19): Raven dereferences `mod_m->mdxm` unconditionally
        // (a null-deref UB path if the handle were bad); this picks the
        // defined fallback of treating a null block as "model not found".
        if let Some(mod_m) = host.model_mdxm(model_list[model_index as usize]) {
            let anim_index = mod_m.anim_index();
            host.model_mdxa(anim_index)
        } else {
            None
        }
    } else {
        // GAP (module doc finding 2): `RE_RegisterModel(fileName)` has no
        // `EngineHost` service — reported upstream; treated the same as
        // `g2_is_paused`'s unresolved by-filename path.
        None
    };

    if mod_a.is_none() {
        return false;
    }

    let mut index = g2_find_bone(mod_a, blist, bone_name);
    if index != -1 && blist[index as usize].flags & BONE_ANGLES_RAGDOLL != 0 {
        return true;
    }
    if index != -1 {
        blist[index as usize].flags &= !BONE_ANGLES_TOTAL;
        blist[index as usize].flags |= flags;
        blist[index as usize].matrix = *matrix;
        blist[index as usize].newMatrix = *matrix;
        return true;
    }

    index = g2_add_bone(mod_a, blist, bone_name);
    if index != -1 {
        blist[index as usize].flags &= !BONE_ANGLES_TOTAL;
        blist[index as usize].flags |= flags;
        blist[index as usize].matrix = *matrix;
        blist[index as usize].newMatrix = *matrix;
        return true;
    }
    // Raven: `assert(0); return qfalse;`
    false
}

/// Raven `int G2_Get_Bone_Index(CGhoul2Info *ghoul2, const char *boneName)` —
/// resolves the anim model **by filename** (`ghoul2->mFileName` through
/// `RE_RegisterModel`/`R_GetModelByHandle`, `:4904-4905`). Unlike
/// `g2_is_paused`'s sibling gap, this function receives the full
/// `CGhoul2Info` — whose `anim_model` was already resolved onto the same file
/// by the last `G2_SetupModelPointers` call — so it reuses that instead of
/// re-deriving a fresh handle from `mFileName` (divergence, reported
/// upstream: behaviorally equivalent whenever the file hasn't reloaded since
/// setup, which is the live path; `host` is threaded per the pinned
/// signature but goes unused).
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:4902-4908`
pub fn g2_get_bone_index(host: &mut impl EngineHost, ghoul2: &CGhoul2Info, bone_name: &str) -> i32 {
    let _ = host;
    g2_find_bone(ghoul2.anim_model, &ghoul2.blist, bone_name)
}

/// Raven `qboolean G2_Set_Bone_Angles_Index( boneInfo_v &blist, const int
/// index, const float *angles, const int flags, const Eorientations yaw,
/// pitch, roll, qhandle_t *modelList, const int modelIndex, const int
/// blendTime, const int currentTime)` — bounds-checks `index`, rejects
/// ragdoll bones, rejects `PREMULT`/`POSTMULT` flags (those need
/// `g2_set_bone_angles`'s model lookup), else sets flags/blend timing and
/// forwards to `g2_generate_matrix` with a null model.
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:434-474`
#[allow(clippy::too_many_arguments)]
pub fn g2_set_bone_angles_index(
    blist: &mut Vec<boneInfo_t>,
    index: i32,
    angles: vec3_t,
    flags: i32,
    yaw: Eorientations,
    pitch: Eorientations,
    roll: Eorientations,
    model_list: &[qhandle_t],
    model_index: i32,
    blend_time: i32,
    current_time: i32,
) -> bool {
    // Raven never reads `modelList`/`modelIndex` in this function's body.
    let _ = (model_list, model_index);
    // C++'s `index >= blist.size()` promotes a negative `index` to a huge
    // `size_t`, so this also catches Raven's own `index == -1` sentinel —
    // the `index != -1` check below it never actually sees a negative index.
    if index < 0 || index as usize >= blist.len() || blist[index as usize].boneNumber == -1 {
        // Raven: `assert(0); return qfalse;`
        return false;
    }
    if blist[index as usize].flags & BONE_ANGLES_RAGDOLL != 0 {
        return true;
    }
    if flags & (BONE_ANGLES_PREMULT | BONE_ANGLES_POSTMULT) != 0 {
        // Raven: `assert(0); return qfalse;` — "you CANNOT call this with an
        // index with these kinds of bone overrides".
        return false;
    }
    blist[index as usize].flags &= !BONE_ANGLES_TOTAL;
    blist[index as usize].flags |= flags;
    blist[index as usize].boneBlendStart = current_time;
    blist[index as usize].boneBlendTime = blend_time;
    g2_generate_matrix(
        None,
        blist,
        index,
        angles,
        flags,
        yaw,
        pitch,
        roll,
    );
    true
}

/// Raven `qboolean G2_Set_Bone_Angles_Matrix_Index(boneInfo_v &blist, const
/// int index, const mdxaBone_t &matrix, const int flags, qhandle_t
/// *modelList, const int modelIndex, const int blendTime, const int
/// currentTime)` — bounds-checks `index`, rejects ragdoll bones, else copies
/// `matrix` into the slot's `matrix`/`newMatrix`.
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:536-564`
#[allow(clippy::too_many_arguments)]
pub fn g2_set_bone_angles_matrix_index(
    blist: &mut Vec<boneInfo_t>,
    index: i32,
    matrix: &mdxaBone_t,
    flags: i32,
    model_list: &[qhandle_t],
    model_index: i32,
    blend_time: i32,
    current_time: i32,
) -> bool {
    let _ = (model_list, model_index);
    if index < 0 || index as usize >= blist.len() || blist[index as usize].boneNumber == -1 {
        return false;
    }
    if blist[index as usize].flags & BONE_ANGLES_RAGDOLL != 0 {
        return true;
    }
    blist[index as usize].flags &= !BONE_ANGLES_TOTAL;
    blist[index as usize].flags |= flags;
    blist[index as usize].boneBlendStart = current_time;
    blist[index as usize].boneBlendTime = blend_time;
    blist[index as usize].matrix = *matrix;
    blist[index as usize].newMatrix = *matrix;
    true
}

/// Raven `qboolean G2_Stop_Bone_Anim_Index(boneInfo_v &blist, const int
/// index)` — bounds-checks `index`, clears the anim flags, forwards to
/// `g2_remove_bone_index`.
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:993-1006`
pub fn g2_stop_bone_anim_index(blist: &mut Vec<boneInfo_t>, index: i32) -> bool {
    if index < 0 || index as usize >= blist.len() || blist[index as usize].boneNumber == -1 {
        return false;
    }
    blist[index as usize].flags &= !BONE_ANIM_TOTAL;
    g2_remove_bone_index(blist, index)
}

/// Raven `qboolean G2_Stop_Bone_Angles_Index(boneInfo_v &blist, const int
/// index)` — bounds-checks `index`, clears the angle-override flags, forwards
/// to `g2_remove_bone_index`.
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:1028-1042`
pub fn g2_stop_bone_angles_index(blist: &mut Vec<boneInfo_t>, index: i32) -> bool {
    if index < 0 || index as usize >= blist.len() || blist[index as usize].boneNumber == -1 {
        return false;
    }
    blist[index as usize].flags &= !BONE_ANGLES_TOTAL;
    g2_remove_bone_index(blist, index)
}

/// Raven `qboolean G2_Set_Bone_Anim_Index(boneInfo_v &blist, const int index,
/// const int startFrame, endFrame, flags, const float animSpeed, const int
/// currentTime, const float setFrame, const int blendTime, const int
/// numFrames)` — bounds-checks `index`, rejects ragdoll bones, computes
/// blend-frame bookkeeping (via `g2_get_bone_anim_index` when
/// `BONE_ANIM_BLEND` is set), then writes the anim frame range/speed/flags
/// and start time.
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:624-812`
#[allow(clippy::too_many_arguments)]
pub fn g2_set_bone_anim_index(
    blist: &mut Vec<boneInfo_t>,
    index: i32,
    start_frame: i32,
    end_frame: i32,
    flags: i32,
    anim_speed: f32,
    current_time: i32,
    set_frame: f32,
    blend_time: i32,
    num_frames: i32,
) -> bool {
    let mut mod_flags = flags;

    if index < 0 || index as usize >= blist.len() || blist[index as usize].boneNumber == -1 {
        // Raven: `assert(0); return qfalse;`
        return false;
    }
    let idx = index as usize;

    if blist[idx].flags & BONE_ANGLES_RAGDOLL != 0 {
        return true;
    }
    // Mark this bone for needing a transform for the cached trace transform stuff.
    blist[idx].flags |= BONE_NEED_TRANSFORM;

    // Raven: `if (setFrame != -1) { assert((setFrame >= startFrame) &&
    // (setFrame <= endFrame)); }` — debug-only sanity check, no runtime effect.

    if flags & BONE_ANIM_BLEND != 0 {
        // Raven shadows `currentFrame`/`animSpeed`/`startFrame`/`endFrame`/
        // `flags` with fresh locals inside this block (`G2_bones.cpp:663-664`)
        // — the outer `start_frame`/`end_frame`/`flags`/`anim_speed`
        // parameters are untouched until after this block closes.
        let mut inner_current_frame = 0.0f32;
        let mut inner_start_frame = 0i32;
        let mut inner_end_frame = 0i32;
        let mut inner_flags = 0i32;
        let mut inner_anim_speed = 0.0f32;
        let got = g2_get_bone_anim_index(
            blist.as_slice(),
            index,
            current_time,
            &mut inner_current_frame,
            &mut inner_start_frame,
            &mut inner_end_frame,
            &mut inner_flags,
            &mut inner_anim_speed,
            &[],
            num_frames,
        );
        if got {
            if blist[idx].blendStart == current_time {
                // Replacing a blend in progress which hasn't started.
                blist[idx].blendTime = blend_time;
            } else {
                if inner_anim_speed < 0.0 {
                    blist[idx].blendFrame = inner_current_frame.floor();
                    blist[idx].blendLerpFrame = inner_current_frame.floor() as i32;
                } else {
                    blist[idx].blendFrame = inner_current_frame;
                    blist[idx].blendLerpFrame = (inner_current_frame + 1.0) as i32;

                    // Cope with if the lerp frame is actually off the end of the anim.
                    if blist[idx].blendFrame >= inner_end_frame as f32 {
                        if blist[idx].flags & BONE_ANIM_OVERRIDE_LOOP != 0 {
                            blist[idx].blendFrame = inner_start_frame as f32;
                        } else if inner_end_frame <= 0 {
                            blist[idx].blendLerpFrame = 0;
                        } else {
                            blist[idx].blendFrame = (inner_end_frame - 1) as f32;
                        }
                    }
                    if blist[idx].blendLerpFrame >= inner_end_frame {
                        if blist[idx].flags & BONE_ANIM_OVERRIDE_LOOP != 0 {
                            blist[idx].blendLerpFrame = inner_start_frame;
                        } else if inner_end_frame <= 0 {
                            blist[idx].blendLerpFrame = 0;
                        } else {
                            blist[idx].blendLerpFrame = inner_end_frame - 1;
                        }
                    }
                }
                blist[idx].blendTime = blend_time;
                blist[idx].blendStart = current_time;
            }
        } else {
            // We weren't animating on this bone — disable the blend.
            blist[idx].blendFrame = 0.0;
            blist[idx].blendLerpFrame = 0;
            blist[idx].blendTime = 0;
            mod_flags &= !BONE_ANIM_BLEND;
        }
    } else {
        blist[idx].blendFrame = 0.0;
        blist[idx].blendLerpFrame = 0;
        blist[idx].blendTime = 0;
        blist[idx].blendStart = 0;
        mod_flags &= !BONE_ANIM_BLEND;
    }

    blist[idx].endFrame = end_frame;
    blist[idx].startFrame = start_frame;
    blist[idx].animSpeed = anim_speed;
    blist[idx].pauseTime = 0;
    if set_frame != -1.0 {
        let value = current_time as f64
            - (((set_frame as f64 - start_frame as f64) * 50.0) / anim_speed as f64);
        blist[idx].lastTime = value as i32;
        blist[idx].startTime = value as i32;
    } else {
        blist[idx].lastTime = current_time;
        blist[idx].startTime = current_time;
    }
    blist[idx].flags &= !BONE_ANIM_TOTAL;
    if blist[idx].flags < 0 {
        blist[idx].flags = 0;
    }
    blist[idx].flags |= mod_flags;

    true
}

/// Raven `qboolean G2_Get_Bone_Anim_Index(boneInfo_v &blist, const int index,
/// const int currentTime, float *currentFrame, int *startFrame, *endFrame,
/// *flags, float *retAnimSpeed, qhandle_t *modelList, int numFrames)` —
/// bounds-checks `index`; if animating, calls the render-side `G2_TimingModel`
/// (canonical `g2_timing_model` in `render/bone_transform.rs`, `tr_ghoul2.cpp:
/// 1167` — cross-file call, module doc findings 4/5) to
/// derive the lerped current frame and writes all out-params, else zeroes
/// them and returns `qfalse`. `model_list` is present in the oracle signature
/// but **never read** in the body (`G2_bones.cpp:872-901`) — kept for 1:1
/// signature fidelity.
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:872-901`
#[allow(clippy::too_many_arguments)]
pub fn g2_get_bone_anim_index(
    blist: &[boneInfo_t],
    index: i32,
    current_time: i32,
    current_frame: &mut f32,
    start_frame: &mut i32,
    end_frame: &mut i32,
    flags: &mut i32,
    ret_anim_speed: &mut f32,
    model_list: &[qhandle_t],
    num_frames: i32,
) -> bool {
    // `model_list` is never read in the body (kept for signature fidelity);
    // `num_frames` feeds the canonical `g2_timing_model`'s `numFramesInFile`
    // param, which only backs debug-only bounds `assert`s (skipped, matching
    // this crate's assert-skipping convention).
    let _ = model_list;
    if index >= 0 && (index as usize) < blist.len() && blist[index as usize].boneNumber != -1 {
        let bone = &blist[index as usize];
        if bone.flags & (BONE_ANIM_OVERRIDE_LOOP | BONE_ANIM_OVERRIDE) != 0 {
            // The canonical `g2_timing_model` takes `&mut boneInfo_t` to perform
            // Raven's one flags-clear write; this call chain's pinned read-only
            // `blist` must not observe it, so hand it a throwaway clone (module
            // doc finding 5) and keep reading the untouched `blist` entry below.
            // Raven seeds all three refs to 0 here, so the (unused) `new_frame`
            // result and the flags-clear arm's pass-through match exactly.
            let mut scratch = bone.clone();
            let (local_current_frame, _local_new_frame, local_lerp) =
                g2_timing_model(&mut scratch, current_time, num_frames, 0, 0, 0.0);
            *current_frame = local_current_frame as f32 + local_lerp;
            *start_frame = bone.startFrame;
            *end_frame = bone.endFrame;
            *flags = bone.flags;
            *ret_anim_speed = bone.animSpeed;
            return true;
        }
    }
    *start_frame = 0;
    *end_frame = 1;
    *current_frame = 0.0;
    *flags = 0;
    *ret_anim_speed = 0.0;
    false
}
