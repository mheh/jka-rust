//! `G2API` ragdoll+IK — the server-live RagDoll/IK solver's `G2API_*`
//! syscall-switch surface (`G2SV-D3`: the 12 ragdoll/IK entry points are all
//! `SV_GameSystemCalls` arms, `sv_game.cpp:1497,1509,1532,1554,1561,1563,1565,
//! 1567,1569,1571,1574,1576`, so the solver runs server-side).
//!
//! Per `docs/subsystems/ghoul2-server.md` roster (`api_ragdoll.rs`, class
//! "G2API ragdoll+IK"): `SetRagDoll`/`ResetRagDoll`/`AnimateG2Models`(rag),
//! `RagPCJConstraint`/`RagPCJGradientSpeed`/`RagEffectorGoal`/`GetRagBonePos`/
//! `RagEffectorKick`/`RagForceSolve`, `SetBoneIKState`/`IKMove`,
//! `AbsurdSmoothing`. `params` on the animate/set entry points is the
//! `RagDollUpdateParams`/`CRagDollParams` types (`ragdoll_update_params.rs`,
//! `gore/crag_doll_params.rs`).
//!
//! Every `G2API_*` entry keeps its 1:1 signature (`G2SV-D6`) and threads
//! `g2: &mut Ghoul2System` (ruling 4/11, state threaded not reached) to
//! resolve the `CGhoul2Info_v` handle through `Ghoul2System.info_array`.
//! `host: &mut impl EngineHost` is added only where the completed body
//! actually reaches a host service: `g2api_set_ragdoll` (`G2_SetRagDoll` ->
//! `G2_GetModA` model-memory read + the `broadsword` cvar read + the
//! unconditional `flrand` PCJ-bone seed inside `G2_Set_Bone_Angles_Rag`,
//! ruling 11/21/36, `G2SV-D15`), `g2api_set_bone_ik_state` (`G2_SetBoneIKState`
//! -> `G2_GetModA` + `G2_ConstructGhoulSkeleton`/`G2_RagDollSetup`, both of
//! which reach `EngineHost` themselves), and `g2api_ik_move` (`G2_IKMove`'s
//! live `#else` arm -> `G2_RagDollSetup`). The remaining entries only flip
//! flags/vectors on a single `CGhoul2Info` and take no `host`.
//!
//! Out-param classification follows the frozen discriminator ("Out-param
//! contract for the un-illustrated `G2API_*` functions", `G2SV-D1`
//! generalized): `g2api_get_rag_bone_pos` is the doc's own cited
//! write-on-success-only archetype alongside `GetBoneAnim`/`GetAnimRange` (its
//! sole body is `{ return qfalse; }`, so the `pos` out-param is never
//! touched on any observed path) and maps to `Option<vec3_t>`, not a
//! write-through `&mut` out-param.
//!
//! **Private-helper colocation (per `ragdoll.rs`'s own module doc, "Scope
//! boundary vs `api_ragdoll.rs`"):** `G2_SetRagDoll`/`G2_ResetRagDoll`/
//! `G2_SetBoneIKState`/`G2_IKMove` are physically defined in `G2_bones.cpp`
//! but roster to this file, and so do their exclusive private setup helpers —
//! `G2_GetModA`, `G2_Find_Bone_Rag`, `G2_Set_Bone_Rag`,
//! `G2_Set_Bone_Angles_Rag`, `G2_Set_Bone_Anim_No_BS`, `G2_InitIK`,
//! `G2_Set_Bone_Angles_IK` — all transcribed below as private `fn`s. The
//! `RAG_*`/`GHOUL2_RAG_*` bit-flag `#define`s these helpers need are
//! `G2_bones.cpp`-file-scope in the oracle (not a shared header); per the
//! established convention in this crate (`api_models.rs`'s own local
//! `GHOUL2_NEWORIGIN` const) they are defined here as private `const`s.
//! **Reported upstream**: `ragdoll.rs`'s `G2_RagDollSetup`/settle-pass bodies
//! will need the same `RAG_PCJ`/`RAG_PCJ_PELVIS`/`RAG_PCJ_MODEL_ROOT`/
//! `RAG_EFFECTOR`/`RAG_WAS_NOT_RENDERED`/`RAG_WAS_EVER_RENDERED` constants;
//! this file does not define the two this file never reads
//! (`RAG_WAS_NOT_RENDERED`/`RAG_WAS_EVER_RENDERED`) — a future porter should
//! consolidate rather than re-`#define` them.
//!
//! **`split_info`/raw-pointer note (reported upstream):** two frozen sibling
//! signatures this file must call take `&mut Ghoul2System` **and** a
//! `CGhoul2Info` reference at once — `render/skeleton.rs`'s
//! `g2_get_bone_matrix_low(g2: &mut Ghoul2System, ghoul2: &CGhoul2Info, ...)`
//! and `ragdoll.rs`'s `g2_rag_doll_setup(g2: &mut Ghoul2System, host, ghoul2:
//! &mut CGhoul2Info, ...)` (the one function in that file taking a raw
//! `CGhoul2Info` instead of the cheap `&mut CGhoul2Info_v` wrapper every
//! sibling entry point uses). Since `CGhoul2Info` instances live inside
//! `Ghoul2System.info_array`, calling either signature safely is impossible
//! without splitting the borrow; `split_info` does that via a raw pointer,
//! mirroring the `root_bone_list` precedent already in this crate
//! (`render/bone_cache.rs`: "internal-only aliasing... a safe borrow here is
//! a lifetime question the doc leaves to the per-file body", §A1). Sound as
//! long as the callee never re-enters `Ghoul2System.info_array` for the same
//! model index while the split pointer is live.

use mp_host_interface::EngineHost;
use mp_qshared::common::mp::ghoul2::{
    BONE_ANGLES_IK, BONE_ANGLES_POSTMULT, BONE_ANGLES_PREMULT, BONE_ANGLES_RAGDOLL,
    BONE_ANGLES_TOTAL, BONE_ANIM_BLEND, BONE_ANIM_OVERRIDE_FREEZE, BONE_ANIM_TOTAL,
};
use mp_qshared::common::mp::qcommon::{sharedRagDollUpdateParams_t, sharedSetBoneIKStateParams_t};
use mp_qshared::shared::{
    mdxaBone_t, qfalse, qtrue, sharedEIKMoveState, sharedERagPhase, sharedIKMoveParams_t, vec3_t,
};

use crate::ghoul2_system::Ghoul2System;
use crate::gore::crag_doll_params::CRagDollParams;
use mp_host_interface::mdx::mdxa::MdxaRef;
use crate::ragdoll_update_params::{RagDollUpdateKind, RagDollUpdateParams};
use crate::shared::cghoul2_info::CGhoul2Info;
use crate::shared::cghoul2_info_v::CGhoul2Info_v;
use crate::{api_collision, bones, misc, ragdoll, render::skeleton};

// ---------------------------------------------------------------------------
// Private bit-flag consts (`G2_bones.cpp`-file-scope `#define`s the oracle
// never puts in a shared header; see the module-doc note above).
// ---------------------------------------------------------------------------

/// Raven `#define GHOUL2_CRAZY_SMOOTH 0x2000` (`G2_local.h:12`).
const GHOUL2_CRAZY_SMOOTH: i32 = 0x2000;
/// Raven `#define GHOUL2_RAG_STARTED 0x0010` (`G2_bones.cpp:1205`).
const GHOUL2_RAG_STARTED: i32 = 0x0010;
/// Raven `#define GHOUL2_RAG_PENDING 0x0100` (`G2_bones.cpp:1206`).
const GHOUL2_RAG_PENDING: i32 = 0x0100;
/// Raven `#define GHOUL2_RAG_DONE 0x0200` (`G2_bones.cpp:1207`).
const GHOUL2_RAG_DONE: i32 = 0x0200;
/// Raven `#define GHOUL2_RAG_COLLISION_DURING_DEATH 0x0400` (`G2_bones.cpp:1208`).
const GHOUL2_RAG_COLLISION_DURING_DEATH: i32 = 0x0400;
/// Raven `#define GHOUL2_RAG_COLLISION_SLIDE 0x0800` (`G2_bones.cpp:1209`).
const GHOUL2_RAG_COLLISION_SLIDE: i32 = 0x0800;
/// Raven `#define GHOUL2_RAG_FORCESOLVE 0x1000` (`G2_bones.cpp:1210`).
const GHOUL2_RAG_FORCESOLVE: i32 = 0x1000;

/// Raven `#define RAG_PCJ (0x00001)` (`G2_bones.cpp:1211`).
const RAG_PCJ: i32 = 0x00001;
/// Raven `#define RAG_PCJ_POST_MULT (0x00002)` (`G2_bones.cpp:1212`).
const RAG_PCJ_POST_MULT: i32 = 0x00002;
/// Raven `#define RAG_PCJ_MODEL_ROOT (0x00004)` (`G2_bones.cpp:1213`).
const RAG_PCJ_MODEL_ROOT: i32 = 0x00004;
/// Raven `#define RAG_PCJ_PELVIS (0x00008)` (`G2_bones.cpp:1214`).
const RAG_PCJ_PELVIS: i32 = 0x00008;
/// Raven `#define RAG_EFFECTOR (0x00100)` (`G2_bones.cpp:1215`).
const RAG_EFFECTOR: i32 = 0x00100;
/// Raven `#define RAG_BONE_LIGHTWEIGHT (0x04000)` (`G2_bones.cpp:1218`).
const RAG_BONE_LIGHTWEIGHT: i32 = 0x04000;
/// Raven `#define RAG_PCJ_IK_CONTROLLED (0x08000)` (`G2_bones.cpp:1219`).
const RAG_PCJ_IK_CONTROLLED: i32 = 0x08000;
/// Raven `#define RAG_UNSNAPPABLE (0x10000)` (`G2_bones.cpp:1220`).
const RAG_UNSNAPPABLE: i32 = 0x10000;

/// Identity `mdxaBone_t` (`G2_bones.cpp:1412-1417`/`4573-4578`, the
/// `static mdxaBone_t id = {1,0,0,0, 0,1,0,0, 0,0,1,0}` both
/// `G2_Set_Bone_Angles_Rag`/`G2_Set_Bone_Angles_IK` memcpy into
/// `bone.ragOverrideMatrix` on first init.
const IDENTITY_BONE: mdxaBone_t = mdxaBone_t {
    matrix: [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
    ],
};

/// The `G2_Set_Bone_Rag` bone-name list (`G2_bones.cpp:1785-1807`) — the
/// commented-out `rtarsal`/`ltarsal` lines never compiled, so they are not
/// transcribed.
const RAG_BONE_NAMES: &[&str] = &[
    "model_root",
    "pelvis",
    "lower_lumbar",
    "upper_lumbar",
    "thoracic",
    "cranium",
    "rhumerus",
    "lhumerus",
    "rradius",
    "lradius",
    "rfemurYZ",
    "lfemurYZ",
    "rtibia",
    "ltibia",
    "rhand",
    "lhand",
    "rtalus",
    "ltalus",
    "rradiusX",
    "lradiusX",
    "rfemurX",
    "lfemurX",
    "ceyebrow",
];

/// The `G2_Set_Bone_Anim_No_BS` bone-name list (`G2_bones.cpp:1809-1826`);
/// every call shares identical `flags`/`animSpeed`/frame args, varying only
/// by bone name, so `g2api_set_ragdoll` loops this list instead of repeating
/// the call seven times (behavior-preserving, porting-rules §10).
const RAG_ANIM_BONE_NAMES: &[&str] = &[
    "upper_lumbar",
    "lower_lumbar",
    "Motion",
    "lfemurYZ",
    "rfemurYZ",
    "rhumerus",
    "lhumerus",
];

/// One `G2_Set_Bone_Angles_Rag` call's per-bone arguments
/// (`G2_bones.cpp:1828-1930`, the live `#if 1` "new base anim" branch —
/// `fRadScale=0.3`, `sFactLeg=sFactArm=sRadArm=sRadLeg=1.0`, all folded into
/// the literal `radius`/angle values below). `angle_min`/`angle_max` are
/// `None` exactly where Raven omits the arguments (C++ default `vec3_t
/// angleMin=0` — a null pointer, not an array — the falsy branch of
/// `if (angleMin&&angleMax)`).
struct RagAngleBone {
    name: &'static str,
    flags: i32,
    radius: f32,
    angle_min: Option<vec3_t>,
    angle_max: Option<vec3_t>,
    blend_time: i32,
}

/// The 23-entry `G2_Set_Bone_Angles_Rag` table, in oracle call order.
const RAG_ANGLE_BONES: &[RagAngleBone] = &[
    RagAngleBone {
        name: "model_root",
        flags: RAG_PCJ_MODEL_ROOT | RAG_PCJ | RAG_UNSNAPPABLE,
        radius: 3.0,
        angle_min: Some([-90.0, -45.0, -45.0]),
        angle_max: Some([90.0, 45.0, 45.0]),
        blend_time: 100,
    },
    RagAngleBone {
        name: "pelvis",
        flags: RAG_PCJ_PELVIS | RAG_PCJ | RAG_PCJ_POST_MULT | RAG_UNSNAPPABLE,
        radius: 3.0,
        angle_min: Some([-45.0, -45.0, -45.0]),
        angle_max: Some([45.0, 45.0, 45.0]),
        blend_time: 100,
    },
    RagAngleBone {
        name: "lower_lumbar",
        flags: RAG_PCJ | RAG_PCJ_POST_MULT | RAG_UNSNAPPABLE,
        radius: 3.0,
        angle_min: Some([-15.0, -15.0, -15.0]),
        angle_max: Some([15.0, 15.0, 15.0]),
        blend_time: 500,
    },
    RagAngleBone {
        name: "upper_lumbar",
        flags: RAG_PCJ | RAG_PCJ_POST_MULT | RAG_UNSNAPPABLE,
        radius: 3.0,
        angle_min: Some([-15.0, -15.0, -15.0]),
        angle_max: Some([15.0, 15.0, 15.0]),
        blend_time: 500,
    },
    RagAngleBone {
        name: "thoracic",
        flags: RAG_PCJ | RAG_PCJ_POST_MULT | RAG_EFFECTOR | RAG_UNSNAPPABLE,
        radius: 3.6,
        angle_min: Some([-25.0, -25.0, -25.0]),
        angle_max: Some([25.0, 25.0, 25.0]),
        blend_time: 500,
    },
    RagAngleBone {
        name: "cranium",
        flags: RAG_PCJ | RAG_PCJ_POST_MULT | RAG_BONE_LIGHTWEIGHT | RAG_UNSNAPPABLE,
        radius: 1.8,
        angle_min: Some([-10.0, -10.0, -90.0]),
        angle_max: Some([10.0, 10.0, 90.0]),
        blend_time: 500,
    },
    RagAngleBone {
        name: "rhumerus",
        flags: RAG_PCJ | RAG_PCJ_POST_MULT | RAG_BONE_LIGHTWEIGHT | RAG_UNSNAPPABLE,
        radius: 1.2,
        angle_min: Some([-100.0, -40.0, -15.0]),
        angle_max: Some([-15.0, 80.0, 15.0]),
        blend_time: 500,
    },
    RagAngleBone {
        name: "lhumerus",
        flags: RAG_PCJ | RAG_PCJ_POST_MULT | RAG_BONE_LIGHTWEIGHT | RAG_UNSNAPPABLE,
        radius: 1.2,
        angle_min: Some([-50.0, -80.0, -15.0]),
        angle_max: Some([15.0, 40.0, 15.0]),
        blend_time: 500,
    },
    RagAngleBone {
        name: "rradius",
        flags: RAG_PCJ | RAG_PCJ_POST_MULT | RAG_BONE_LIGHTWEIGHT,
        radius: 0.9,
        angle_min: Some([-25.0, -20.0, -20.0]),
        angle_max: Some([90.0, 20.0, -20.0]),
        blend_time: 500,
    },
    RagAngleBone {
        name: "lradius",
        flags: RAG_PCJ | RAG_PCJ_POST_MULT | RAG_BONE_LIGHTWEIGHT,
        radius: 0.9,
        angle_min: Some([-90.0, -20.0, -20.0]),
        angle_max: Some([30.0, 20.0, -20.0]),
        blend_time: 500,
    },
    RagAngleBone {
        name: "rfemurYZ",
        flags: RAG_PCJ | RAG_PCJ_POST_MULT | RAG_BONE_LIGHTWEIGHT,
        radius: 1.8,
        angle_min: Some([-80.0, -50.0, -20.0]),
        angle_max: Some([30.0, 5.0, 20.0]),
        blend_time: 500,
    },
    RagAngleBone {
        name: "lfemurYZ",
        flags: RAG_PCJ | RAG_PCJ_POST_MULT | RAG_BONE_LIGHTWEIGHT,
        radius: 1.8,
        angle_min: Some([-60.0, -5.0, -20.0]),
        angle_max: Some([50.0, 50.0, 20.0]),
        blend_time: 500,
    },
    RagAngleBone {
        name: "rtibia",
        flags: RAG_PCJ | RAG_PCJ_POST_MULT | RAG_EFFECTOR | RAG_BONE_LIGHTWEIGHT,
        radius: 1.2,
        angle_min: Some([-20.0, -15.0, -15.0]),
        angle_max: Some([100.0, 15.0, 15.0]),
        blend_time: 500,
    },
    RagAngleBone {
        name: "ltibia",
        flags: RAG_PCJ | RAG_PCJ_POST_MULT | RAG_EFFECTOR | RAG_BONE_LIGHTWEIGHT,
        radius: 1.2,
        angle_min: Some([20.0, -15.0, -15.0]),
        angle_max: Some([100.0, 15.0, 15.0]),
        blend_time: 500,
    },
    RagAngleBone {
        name: "rhand",
        flags: RAG_EFFECTOR | RAG_BONE_LIGHTWEIGHT,
        radius: 2.16,
        angle_min: None,
        angle_max: None,
        blend_time: 500,
    },
    RagAngleBone {
        name: "lhand",
        flags: RAG_EFFECTOR | RAG_BONE_LIGHTWEIGHT,
        radius: 2.16,
        angle_min: None,
        angle_max: None,
        blend_time: 500,
    },
    RagAngleBone {
        name: "rtalus",
        flags: RAG_EFFECTOR | RAG_BONE_LIGHTWEIGHT,
        radius: 1.44,
        angle_min: None,
        angle_max: None,
        blend_time: 500,
    },
    RagAngleBone {
        name: "ltalus",
        flags: RAG_EFFECTOR | RAG_BONE_LIGHTWEIGHT,
        radius: 1.44,
        angle_min: None,
        angle_max: None,
        blend_time: 500,
    },
    RagAngleBone {
        name: "rradiusX",
        flags: RAG_EFFECTOR | RAG_BONE_LIGHTWEIGHT,
        radius: 2.16,
        angle_min: None,
        angle_max: None,
        blend_time: 500,
    },
    RagAngleBone {
        name: "lradiusX",
        flags: RAG_EFFECTOR | RAG_BONE_LIGHTWEIGHT,
        radius: 2.16,
        angle_min: None,
        angle_max: None,
        blend_time: 500,
    },
    RagAngleBone {
        name: "rfemurX",
        flags: RAG_EFFECTOR | RAG_BONE_LIGHTWEIGHT,
        radius: 3.6,
        angle_min: None,
        angle_max: None,
        blend_time: 500,
    },
    RagAngleBone {
        name: "lfemurX",
        flags: RAG_EFFECTOR | RAG_BONE_LIGHTWEIGHT,
        radius: 3.6,
        angle_min: None,
        angle_max: None,
        blend_time: 500,
    },
    RagAngleBone {
        name: "ceyebrow",
        flags: RAG_EFFECTOR | RAG_BONE_LIGHTWEIGHT,
        radius: 5.0,
        angle_min: None,
        angle_max: None,
        blend_time: 500,
    },
];

/// `G2_InitIK`'s live tail (`G2_bones.cpp:4626-4649`, the `#if 0` block above
/// it never compiles): `pcjFlags=RAG_PCJ|RAG_PCJ_POST_MULT|RAG_EFFECTOR`
/// against every listed bone, `angleMin`/`angleMax` always omitted (`None`).
const IK_INIT_BONES: &[(&str, f32)] = &[
    ("rhand", 6.0),
    ("lhand", 6.0),
    ("rtibia", 4.0),
    ("ltibia", 4.0),
    ("rtalus", 4.0),
    ("ltalus", 4.0),
    ("rradiusX", 6.0),
    ("lradiusX", 6.0),
    ("rfemurX", 10.0),
    ("lfemurX", 10.0),
    ("ceyebrow", 10.0),
];

// ---------------------------------------------------------------------------
// Private helpers (colocated per the module-doc note above).
// ---------------------------------------------------------------------------

/// Splits a live `&mut Ghoul2System` so `model`'s `CGhoul2Info` can be used
/// as an independent pointer alongside further `&mut Ghoul2System` use.
/// See the module-doc "`split_info`/raw-pointer note".
fn split_info(g2: &mut Ghoul2System, ghoul2: &CGhoul2Info_v, model: i32) -> *mut CGhoul2Info {
    ghoul2.get_mut(g2, model) as *mut CGhoul2Info
}

/// Raven `const mdxaHeader_t *G2_GetModA(CGhoul2Info &ghoul2)` — the model's
/// parsed `.gla` block, read off its already-built bone cache; `0`/null if
/// no cache exists yet.
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:591-599`
fn g2_get_mod_a(g2: &Ghoul2System, info: &CGhoul2Info) -> Option<MdxaRef<'static>> {
    match info.bone_cache {
        Some(id) => g2.bone_caches.get(id).and_then(|cache| cache.mdxa),
        None => None,
    }
}

/// Reads bone `bone_number`'s skeleton name out of the loader's parsed
/// `.gla` block (`(byte*)aHeader + sizeof(mdxaHeader_t) + offsets->
/// offsets[boneNumber]`, `G2_bones.cpp:1274-1290`) and case-insensitively
/// compares it (Raven `stricmp`) to `bone_name`. Split out of
/// `g2_find_bone_rag` so the raw-pointer arithmetic is independently
/// testable (`#[cfg(test)]` below) without needing a full `boneInfo_t`/
/// `CGhoul2Info`.
///
/// # Safety invariant (not `unsafe fn`: the pointer is trusted, not the caller)
/// `header` must be null or a valid `EngineHost::model_mdxa` block
/// (`G2SV-D5`: `mdxaHeader_t`/`mdxaSkelOffsets_t`/`mdxaSkel_t` are never
/// named here, so the wire sizes are replicated instead of imported from
/// `mp_renderer::mdx_format`, which this crate may not depend on).
fn skel_bone_name_matches(header: Option<MdxaRef<'static>>, bone_number: i32, bone_name: &str) -> bool {
    let Some(mdxa) = header else {
        return false;
    };
    if bone_number < 0 {
        return false;
    }
    // `header` is trusted valid for the model's lifetime (see the invariant
    // above); `stricmp` is byte-wise case-insensitive.
    mdxa.skel(bone_number).name_matches(bone_name)
}

/// Raven `int G2_Find_Bone_Rag(CGhoul2Info *ghlInfo, boneInfo_v &blist,
/// const char *boneName)` — walks `blist` for the entry whose skeletal name
/// (via `ghlInfo->aHeader`, not `G2_Find_Bone`'s `model_t`-based lookup)
/// matches `bone_name`; `blist` is always `ghlInfo`'s own `mBlist` at every
/// call site, so this takes just `info` (porting-rules §A1, internal-only).
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:1271-1298`
fn g2_find_bone_rag(info: &CGhoul2Info, bone_name: &str) -> i32 {
    for (i, bone) in info.blist.iter().enumerate() {
        if bone.boneNumber == -1 {
            continue;
        }
        if skel_bone_name_matches(info.a_header, bone.boneNumber, bone_name) {
            return i as i32;
        }
    }
    -1
}

/// Raven `static int G2_Set_Bone_Rag(const mdxaHeader_t *mod_a, boneInfo_v
/// &blist, const char *boneName, CGhoul2Info &ghoul2, const vec3_t scale,
/// const vec3_t origin)` — finds/adds `bone_name`, stamps its
/// `extraVec1`/`originalTrueBoneMatrix`/`basepose`/`baseposeInv`/
/// `originalOrigin` from the live bone matrix (`G2_GetBoneMatrixLow`).
/// `mod_a` is unused in the oracle body (kept out of this port, §A1).
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:1339-1359`
fn g2_set_bone_rag(
    g2: &mut Ghoul2System,
    ghoul2: &mut CGhoul2Info_v,
    model: i32,
    world_matrix: &mdxaBone_t,
    bone_name: &str,
    scale: vec3_t,
    origin: vec3_t,
) -> i32 {
    let info_ptr = split_info(g2, ghoul2, model);
    // SAFETY: see `split_info`'s module-doc note; `g2_get_bone_matrix_low`
    // only touches `g2.bone_caches`, never re-entering `info_array`.
    let info = unsafe { &mut *info_ptr };
    let mut index = g2_find_bone_rag(info, bone_name);
    if index == -1 {
        let anim_model = info.anim_model;
        index = bones::g2_add_bone(anim_model, &mut info.blist, bone_name);
    }
    if index != -1 {
        let idx = index as usize;
        info.blist[idx].extraVec1 = origin;
        let bone_number = info.blist[idx].boneNumber;
        let (matrix, basepose, basepose_inv) = skeleton::g2_get_bone_matrix_low(
            g2,
            unsafe { &*info_ptr },
            bone_number,
            scale,
            world_matrix,
        );
        let info = unsafe { &mut *info_ptr };
        let bone = &mut info.blist[idx];
        bone.originalTrueBoneMatrix = matrix;
        bone.basepose = basepose;
        bone.baseposeInv = basepose_inv;
        bone.originalOrigin = [
            matrix.matrix[0][3],
            matrix.matrix[1][3],
            matrix.matrix[2][3],
        ];
    }
    index
}

/// The randomized `currentAngles` seed a fresh non-root/pelvis PCJ bone gets
/// on first init (`G2_bones.cpp:1450-1461`, the live `#else` arm of the
/// `#if 0` — the `#define flrand Q_flrand` at `:1212` is commented out, so
/// this is the real `flrand`/`Q_flrand`, `EngineHost::flrand`): three
/// heavily-central `flrand` products mapped into `[minAngles[k],
/// maxAngles[k]]`. Split out of `g2_set_bone_angles_rag` so the seeding math
/// is independently testable against the deterministic `MockHost` (below)
/// without needing a full `boneInfo_t`.
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:1450-1461`
fn rag_random_angle_seed(host: &mut impl EngineHost, min: vec3_t, max: vec3_t) -> vec3_t {
    let mut angles = [0.0f32; 3];
    for k in 0..3 {
        let mut scalar = host.flrand(-1.0, 1.0);
        scalar *= host.flrand(-1.0, 1.0) * host.flrand(-1.0, 1.0);
        // heavily central distribution, centered on .5 (and small)
        scalar *= 0.5;
        scalar += 0.5;
        angles[k] = (min[k] - max[k]) * scalar + max[k];
    }
    angles
}

/// Raven `static int G2_Set_Bone_Angles_Rag(CGhoul2Info &ghoul2, const
/// mdxaHeader_t *mod_a, boneInfo_v &blist, const char *boneName, const int
/// flags, const float radius, const vec3_t angleMin=0, const vec3_t
/// angleMax=0, const int blendTime=500)` — finds/adds `bone_name`, flags it
/// `BONE_ANGLES_RAGDOLL` (+`PREMULT`/`POSTMULT` per the `RAG_PCJ*` flags;
/// the `assert(!"Invalid RAG PCJ")` debug-only fallback is dropped, F19),
/// and on first init (`!bone.lastTimeUpdated`) resets the ragdoll runtime
/// state to identity/zero and seeds `currentAngles` — either `[0,0,0]`
/// (root/pelvis/non-PCJ bones) or [`rag_random_angle_seed`] otherwise.
/// `mod_a` is unused in the oracle body (kept out of this port, §A1).
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:1349-1479`
#[allow(clippy::too_many_arguments)]
fn g2_set_bone_angles_rag(
    host: &mut impl EngineHost,
    info: &mut CGhoul2Info,
    bone_name: &str,
    flags: i32,
    radius: f32,
    angle_min: Option<vec3_t>,
    angle_max: Option<vec3_t>,
    blend_time: i32,
    cur_time: i32,
) -> i32 {
    let mut index = g2_find_bone_rag(info, bone_name);
    if index == -1 {
        let anim_model = info.anim_model;
        index = bones::g2_add_bone(anim_model, &mut info.blist, bone_name);
    }
    if index == -1 {
        return index;
    }
    let idx = index as usize;
    let needs_init = {
        let bone = &mut info.blist[idx];
        bone.flags &= !BONE_ANGLES_TOTAL;
        bone.flags |= BONE_ANGLES_RAGDOLL;
        if flags & RAG_PCJ != 0 {
            if flags & RAG_PCJ_POST_MULT != 0 {
                bone.flags |= BONE_ANGLES_POSTMULT;
            } else if flags & RAG_PCJ_MODEL_ROOT != 0 {
                bone.flags |= BONE_ANGLES_PREMULT;
            }
            // else: Raven `assert(!"Invalid RAG PCJ")` — debug-only, dropped (F19).
        }
        bone.ragStartTime = cur_time;
        bone.boneBlendStart = bone.ragStartTime;
        bone.boneBlendTime = blend_time;
        bone.radius = radius;
        bone.weight = 1.0;
        bone.epGravFactor = 0.0;
        bone.epVelocity = [0.0, 0.0, 0.0];
        bone.solidCount = 0;
        bone.physicsSettled = false;
        bone.snapped = false;
        bone.parentBoneIndex = -1;
        bone.offsetRotation = 0.0;
        bone.overGradSpeed = 0.0;
        bone.overGoalSpot = [0.0, 0.0, 0.0];
        bone.hasOverGoal = false;
        bone.hasAnimFrameMatrix = -1;
        match (angle_min, angle_max) {
            (Some(min), Some(max)) => {
                bone.minAngles = min;
                bone.maxAngles = max;
            }
            _ => {
                bone.minAngles = bone.currentAngles;
                bone.maxAngles = bone.currentAngles;
            }
        }
        if bone.lastTimeUpdated == 0 {
            bone.ragOverrideMatrix = IDENTITY_BONE;
            bone.anglesOffset = [0.0, 0.0, 0.0];
            bone.positionOffset = [0.0, 0.0, 0.0];
            bone.velocityEffector = [0.0, 0.0, 0.0];
            bone.velocityRoot = [0.0, 0.0, 0.0];
            bone.lastPosition = [0.0, 0.0, 0.0];
            bone.lastShotDir = [0.0, 0.0, 0.0];
            bone.lastContents = 0;
            bone.firstCollisionTime = bone.ragStartTime;
            bone.restTime = 0;
            bone.firstTime = 0;
            bone.RagFlags = flags;
            bone.DependentRagIndexMask = 0;
            true
        } else {
            false
        }
    };
    if needs_init {
        ragdoll::g2_generate_matrix_rag(&mut info.blist, index);
        let bone = &mut info.blist[idx];
        if (flags & (RAG_PCJ_MODEL_ROOT | RAG_PCJ_PELVIS) != 0) || (flags & RAG_PCJ == 0) {
            bone.currentAngles = [0.0, 0.0, 0.0];
        } else {
            let min = bone.minAngles;
            let max = bone.maxAngles;
            bone.currentAngles = rag_random_angle_seed(host, min, max);
        }
        bone.lastAngles = bone.currentAngles;
    }
    index
}

/// Raven `qboolean G2_Set_Bone_Anim_No_BS(CGhoul2Info &ghoul2, const
/// mdxaHeader_t *mod, boneInfo_v &blist, const char *boneName, const int
/// argStartFrame, const int argEndFrame, const int flags, const float
/// animSpeed, const int currentTime, const float setFrame, const int
/// blendTime, const int creationID, bool resetBonemap=true)` — finds/adds
/// `bone_name` and stamps its anim frame range/speed/flags. `mod`/
/// `currentTime`/`setFrame`/`blendTime`/`creationID`/`resetBonemap` are all
/// unused in the oracle body (the `lastTime`/`startTime`/`boneMap` lines
/// that would read them are commented out) — dropped (§A1, internal-only).
/// Faithfully preserves a real oracle quirk: the found-bone branch resets
/// `blendStart=0` but the added-bone branch does not.
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:1522-1567`
fn g2_set_bone_anim_no_bs(
    info: &mut CGhoul2Info,
    bone_name: &str,
    start_frame: i32,
    end_frame: i32,
    flags: i32,
    anim_speed: f32,
) -> bool {
    let mod_flags = flags & !BONE_ANIM_BLEND;
    let index = g2_find_bone_rag(info, bone_name);
    if index != -1 {
        let bone = &mut info.blist[index as usize];
        bone.blendFrame = 0.0;
        bone.blendLerpFrame = 0;
        bone.blendTime = 0;
        bone.blendStart = 0;
        bone.endFrame = end_frame;
        bone.startFrame = start_frame;
        bone.animSpeed = anim_speed;
        bone.pauseTime = 0;
        bone.flags &= !BONE_ANIM_TOTAL;
        bone.flags |= mod_flags;
        return true;
    }
    let anim_model = info.anim_model;
    let index = bones::g2_add_bone(anim_model, &mut info.blist, bone_name);
    if index != -1 {
        let bone = &mut info.blist[index as usize];
        bone.blendFrame = 0.0;
        bone.blendLerpFrame = 0;
        bone.blendTime = 0;
        // Raven's Add_Bone branch omits `blendStart=0` here (only the
        // Find_Bone_Rag branch above sets it) — faithful oracle quirk, not
        // fixed (porting-rules §A2, no speculative behavior).
        bone.endFrame = end_frame;
        bone.startFrame = start_frame;
        bone.animSpeed = anim_speed;
        bone.pauseTime = 0;
        bone.flags &= !BONE_ANIM_TOTAL;
        bone.flags |= mod_flags;
        return true;
    }
    // Raven: `assert(0); return qfalse;`
    false
}

/// Raven `static int G2_Set_Bone_Angles_IK(CGhoul2Info &ghoul2, const
/// mdxaHeader_t *mod_a, boneInfo_v &blist, const char *boneName, const int
/// flags, const float radius, const vec3_t angleMin=0, const vec3_t
/// angleMax=0, const int blendTime=500)` — `G2_Set_Bone_Angles_Rag`'s IK
/// sibling: flags `BONE_ANGLES_IK` (clearing `_RAGDOLL`), no `flrand`
/// seeding on first init (`currentAngles` always zeroed). `mod_a`/
/// `blend_time` are both unused in the oracle body (kept out, §A1).
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:4536-4602`
fn g2_set_bone_angles_ik(
    info: &mut CGhoul2Info,
    bone_name: &str,
    flags: i32,
    radius: f32,
    angle_min: Option<vec3_t>,
    angle_max: Option<vec3_t>,
    cur_time: i32,
) -> i32 {
    let mut index = g2_find_bone_rag(info, bone_name);
    if index == -1 {
        let anim_model = info.anim_model;
        index = bones::g2_add_bone(anim_model, &mut info.blist, bone_name);
    }
    if index == -1 {
        return index;
    }
    let idx = index as usize;
    let needs_init = {
        let bone = &mut info.blist[idx];
        bone.flags |= BONE_ANGLES_IK;
        bone.flags &= !BONE_ANGLES_RAGDOLL;
        bone.ragStartTime = cur_time;
        bone.radius = radius;
        bone.weight = 1.0;
        match (angle_min, angle_max) {
            (Some(min), Some(max)) => {
                bone.minAngles = min;
                bone.maxAngles = max;
            }
            _ => {
                bone.minAngles = bone.currentAngles;
                bone.maxAngles = bone.currentAngles;
            }
        }
        if bone.lastTimeUpdated == 0 {
            bone.ragOverrideMatrix = IDENTITY_BONE;
            bone.anglesOffset = [0.0, 0.0, 0.0];
            bone.positionOffset = [0.0, 0.0, 0.0];
            bone.velocityEffector = [0.0, 0.0, 0.0];
            bone.velocityRoot = [0.0, 0.0, 0.0];
            bone.lastPosition = [0.0, 0.0, 0.0];
            bone.lastShotDir = [0.0, 0.0, 0.0];
            bone.lastContents = 0;
            bone.firstCollisionTime = bone.ragStartTime;
            bone.restTime = 0;
            bone.firstTime = 0;
            bone.RagFlags = flags;
            bone.DependentRagIndexMask = 0;
            true
        } else {
            false
        }
    };
    if needs_init {
        ragdoll::g2_generate_matrix_rag(&mut info.blist, index);
        let bone = &mut info.blist[idx];
        bone.currentAngles = [0.0, 0.0, 0.0];
        bone.lastAngles = bone.currentAngles;
    }
    index
}

/// Raven `void G2_InitIK(CGhoul2Info_v &ghoul2V, sharedRagDollUpdateParams_t
/// *parms, int time, const mdxaHeader_t *mod_a, int model)` — rebuilds the
/// skeleton then flags the effector-only bone set `RAG_PCJ|
/// RAG_PCJ_POST_MULT|RAG_EFFECTOR` via [`g2_set_bone_angles_ik`]. The
/// leading `#if 0` PCJ-bone block (`:4614-4644`) never compiles; only the
/// live tail (`IK_INIT_BONES`) is transcribed. `mod_a` is unused in the live
/// body (kept out, §A1).
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:4609-4662`
fn g2_init_ik(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghoul2: &mut CGhoul2Info_v,
    parms: &sharedRagDollUpdateParams_t,
    time: i32,
    model: i32,
) {
    let (_world_matrix, _world_matrix_inv) =
        misc::g2_generate_world_matrix(parms.angles, parms.position);
    skeleton::g2_construct_ghoul_skeleton(g2, host, ghoul2, time, false, parms.scale);

    const PCJ_FLAGS: i32 = RAG_PCJ | RAG_PCJ_POST_MULT | RAG_EFFECTOR;
    let info = ghoul2.get_mut(g2, model);
    for (name, radius) in IK_INIT_BONES {
        g2_set_bone_angles_ik(info, name, PCJ_FLAGS, *radius, None, None, time);
    }
}

/// Raven `static void G2_RagDollMatchPosition()` (`G2_bones.cpp:1491-1520`).
///
/// **Reported upstream**: `ragdoll.rs`'s module doc (dead-declaration finding
/// #2) misclassifies this as dead, checking only the also-dead
/// `G2_SetRagDollBullet` caller (`:2035` in that finding's numbering); it is
/// genuinely reached from the LIVE `G2_SetRagDoll` at the real `:2035`
/// (this file's `g2api_set_ragdoll`, the settle loop below). Ported here —
/// not in `ragdoll.rs`, which this porter must not touch — because its sole
/// live caller is this file. Walks the solver's solve-order rag bones,
/// reconstructed via each `boneInfo_t::ragIndex` (the frozen per-bone field
/// `G2_RagDollSetup`'s second pass already stamps): `RagDollSolver` carries
/// no separate solve-order array (`G2SV-D13`(b)/ruling 29 leaves that
/// lookup to the bone-number-keyed `rag`/`blist_index` fields), so this
/// reconstructs the oracle's `ragBoneData[i]`/`ragEffectors[i]` mapping via
/// a linear `ragIndex` scan instead of inventing a new array field. The
/// permanently-dead `if (0&&...)` pelvis branch (`:1500-1507`) is not
/// transcribed.
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:1491-1520`
fn g2_rag_doll_match_position(g2: &mut Ghoul2System, info: &mut CGhoul2Info) {
    g2.rag.have_desired_pelvis_offset = false;
    let num_rags = g2.rag.num_rags;
    for i in 0..num_rags {
        let Some(bone) = info
            .blist
            .iter_mut()
            .find(|b| b.boneNumber >= 0 && b.ragIndex == i)
        else {
            continue;
        };
        let Some(e) = g2.rag.effectors.get_mut(i as usize) else {
            continue;
        };
        if bone.RagFlags & RAG_EFFECTOR == 0 {
            continue;
        }
        e.desired_origin = bone.originalOrigin;
        let mut dir = [0.0f32; 3];
        for k in 0..3 {
            dir[k] = e.desired_origin[k] - e.current_origin[k];
        }
        e.desired_direction = dir;
        bone.lastPosition = e.current_origin;
    }
}

/// Raven `qboolean G2_RagDollSetup(...)` call wrapper — splits `model`'s
/// `CGhoul2Info` out of `Ghoul2System` so both it and `g2` can be passed to
/// `ragdoll.rs`'s `g2_rag_doll_setup` at once (see the module-doc
/// "`split_info`/raw-pointer note").
fn call_rag_doll_setup(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghoul2: &mut CGhoul2Info_v,
    model: i32,
    frame_num: i32,
    reset_origin: bool,
    origin: vec3_t,
    any_rendered: bool,
) -> bool {
    let info_ptr = split_info(g2, ghoul2, model);
    // SAFETY: see `split_info`'s module-doc note.
    ragdoll::g2_rag_doll_setup(
        g2,
        host,
        unsafe { &mut *info_ptr },
        frame_num,
        reset_origin,
        origin,
        any_rendered,
    )
}

// ---------------------------------------------------------------------------
// The frozen `G2API_*` public surface.
// ---------------------------------------------------------------------------

/// Raven `void G2API_AbsurdSmoothing(CGhoul2Info_v &ghoul2, qboolean status)`
/// — set/clear `GHOUL2_CRAZY_SMOOTH` on `ghoul2[0].mFlags`.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:1386-1399`
pub fn g2api_absurd_smoothing(g2: &mut Ghoul2System, ghoul2: &mut CGhoul2Info_v, status: bool) {
    let info = ghoul2.get_mut(g2, 0);
    if status {
        info.flags |= GHOUL2_CRAZY_SMOOTH;
    } else {
        info.flags &= !GHOUL2_CRAZY_SMOOTH;
    }
}

/// Raven `void G2API_SetRagDoll(CGhoul2Info_v &ghoul2, CRagDollParams *parms)`
/// -> `G2_SetRagDoll` (`G2_bones.cpp:1622`). Reads `broadsword`/
/// `broadsword_waitforshot`/`broadsword_dontstopanim` (`EngineHost::cvar_integer`,
/// ruling 36) and the model's `mdxaHeader_t` via `G2_GetModA`
/// (`EngineHost::model_mdxa`, `G2SV-D5`/`G2SV-D15`), and — through
/// `G2_Set_Bone_Angles_Rag`'s per-PCJ-bone init, no `DEDICATED` guard —
/// unconditionally calls `flrand` (`:1468-1469`, `EngineHost::flrand`,
/// ruling 11/21).
///
/// Frozen verbatim in `docs/subsystems/ghoul2-server.md` `## Seam definition`.
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:1404-1407`, `G2_bones.cpp:1622-1782`
pub fn g2api_set_ragdoll(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghoul2: &mut CGhoul2Info_v,
    parms: &mut CRagDollParams,
) {
    // `!parms` is dead in this port: the frozen signature already collapsed
    // Raven's nullable `CRagDollParams *parms` to a plain `&mut` reference.
    parms.CallRagDollBegin = qfalse;
    if host.cvar_integer("broadsword") == 0 {
        return;
    }

    let count = ghoul2.size(g2);
    let mut model = 0;
    while model < count {
        if ghoul2.get(g2, model).modelindex != -1 {
            break;
        }
        model += 1;
    }
    if model == count {
        return;
    }

    let mod_a = g2_get_mod_a(g2, ghoul2.get(g2, model));
    if mod_a.is_none() {
        return;
    }
    let cur_time = api_collision::g2api_get_time(g2, 0);
    let index = g2_find_bone_rag(ghoul2.get(g2, model), "model_root");

    // The `#ifndef DEDICATED` `ERagPhase` switch (`G2_bones.cpp:1653-1774`);
    // the WinDed DEDICATED build's live arm is this whole block.
    match parms.RagPhase {
        sharedERagPhase::RP_START_DEATH_ANIM => {
            ghoul2.get_mut(g2, model).flags |= GHOUL2_RAG_PENDING;
            return;
        }
        sharedERagPhase::RP_END_DEATH_ANIM => {
            ghoul2.get_mut(g2, model).flags |= GHOUL2_RAG_PENDING | GHOUL2_RAG_DONE;
            let waitforshot = host.cvar_integer("broadsword_waitforshot");
            if waitforshot != 0 {
                if waitforshot == 2 {
                    let flags = ghoul2.get(g2, model).flags;
                    if flags & (GHOUL2_RAG_COLLISION_DURING_DEATH | GHOUL2_RAG_COLLISION_SLIDE) == 0
                    {
                        return;
                    }
                } else {
                    return;
                }
            }
        }
        sharedERagPhase::RP_DEATH_COLLISION => {
            let info = ghoul2.get_mut(g2, model);
            if parms.collisionType != 0 {
                info.flags |= GHOUL2_RAG_COLLISION_SLIDE;
            } else {
                info.flags |= GHOUL2_RAG_COLLISION_DURING_DEATH;
            }
            let dontstopanim = host.cvar_integer("broadsword_dontstopanim");
            let waitforshot = host.cvar_integer("broadsword_waitforshot");
            if dontstopanim != 0 || waitforshot != 0 {
                if ghoul2.get(g2, model).flags & GHOUL2_RAG_DONE == 0 {
                    return;
                }
            }
        }
        sharedERagPhase::RP_CORPSE_SHOT => {
            // The live body (kick-strength/velocity apply) is itself a
            // stale commented-out block in the oracle
            // (`G2_bones.cpp:1698-1716`, "Would need ent pointer here" —
            // rww's own note that SP-only corpse-shot never shipped for
            // this mode); nothing observable happens.
        }
        sharedERagPhase::RP_GET_PELVIS_OFFSET | sharedERagPhase::RP_SET_PELVIS_OFFSET => {
            if parms.RagPhase == sharedERagPhase::RP_GET_PELVIS_OFFSET {
                parms.pelvisAnglesOffset = [0.0, 0.0, 0.0];
                parms.pelvisPositionOffset = [0.0, 0.0, 0.0];
            }
            if index >= 0 && (index as usize) < ghoul2.get(g2, model).blist.len() {
                let info = ghoul2.get_mut(g2, model);
                let bone = &mut info.blist[index as usize];
                if bone.boneNumber >= 0 && bone.flags & BONE_ANGLES_RAGDOLL != 0 {
                    if parms.RagPhase == sharedERagPhase::RP_GET_PELVIS_OFFSET {
                        parms.pelvisAnglesOffset = bone.anglesOffset;
                        parms.pelvisPositionOffset = bone.positionOffset;
                    } else {
                        bone.anglesOffset = parms.pelvisAnglesOffset;
                        bone.positionOffset = parms.pelvisPositionOffset;
                    }
                }
            }
            return;
        }
        sharedERagPhase::RP_DISABLE_EFFECTORS => {
            // not doing anything with this yet
            return;
        }
    }

    if ghoul2.get(g2, model).flags & GHOUL2_RAG_STARTED != 0 {
        // only going to begin ragdoll once
        return;
    }

    ghoul2.get_mut(g2, model).flags |= GHOUL2_RAG_PENDING | GHOUL2_RAG_DONE | GHOUL2_RAG_STARTED;
    parms.CallRagDollBegin = qtrue;

    let (world_matrix, _world_matrix_inv) =
        misc::g2_generate_world_matrix(parms.angles, parms.position);
    skeleton::g2_construct_ghoul_skeleton(g2, host, ghoul2, cur_time, false, parms.scale);

    for name in RAG_BONE_NAMES {
        g2_set_bone_rag(
            g2,
            ghoul2,
            model,
            &world_matrix,
            name,
            parms.scale,
            parms.position,
        );
    }

    let start_frame = parms.startFrame;
    let end_frame = parms.endFrame;
    for name in RAG_ANIM_BONE_NAMES {
        let info = ghoul2.get_mut(g2, model);
        g2_set_bone_anim_no_bs(
            info,
            name,
            start_frame,
            end_frame - 1,
            BONE_ANIM_OVERRIDE_FREEZE | BONE_ANIM_BLEND,
            1.0,
        );
    }

    // "should already be set" (Raven's own comment) — re-run anyway.
    skeleton::g2_construct_ghoul_skeleton(g2, host, ghoul2, cur_time, false, parms.scale);

    for entry in RAG_ANGLE_BONES {
        let info = ghoul2.get_mut(g2, model);
        g2_set_bone_angles_rag(
            host,
            info,
            entry.name,
            entry.flags,
            entry.radius,
            entry.angle_min,
            entry.angle_max,
            entry.blend_time,
            cur_time,
        );
    }

    if !call_rag_doll_setup(
        g2,
        host,
        ghoul2,
        model,
        cur_time,
        true,
        parms.position,
        false,
    ) {
        // Raven: `assert(!"failed to add any rag bones"); return;`
        return;
    }
    ragdoll::g2_rag_doll_current_position(
        g2,
        host,
        ghoul2,
        model,
        cur_time,
        parms.angles,
        parms.position,
        parms.scale,
    );

    let mut fparms = RagDollUpdateParams {
        angles: parms.angles,
        position: parms.position,
        scale: parms.scale,
        velocity: [0.0, 0.0, 0.0],
        me: parms.me,
        settle_frame: parms.endFrame,
        kind: RagDollUpdateKind::Server,
    };

    // "Guess I don't need to do this, do I?" (Raven's own comment) — re-run anyway.
    skeleton::g2_construct_ghoul_skeleton(g2, host, ghoul2, cur_time, false, parms.scale);

    let d_pos = parms.position;
    for k in 0..20i32 {
        ragdoll::g2_rag_doll_settle_position_numero_trois(
            g2,
            host,
            ghoul2,
            d_pos,
            Some(&mut fparms),
            cur_time,
        );
        ragdoll::g2_rag_doll_current_position(
            g2,
            host,
            ghoul2,
            model,
            cur_time,
            parms.angles,
            d_pos,
            parms.scale,
        );
        {
            let info_ptr = split_info(g2, ghoul2, model);
            // SAFETY: see `split_info`'s module-doc note.
            g2_rag_doll_match_position(g2, unsafe { &mut *info_ptr });
        }
        ragdoll::g2_rag_doll_solve(
            g2,
            host,
            ghoul2,
            model,
            1.0 - (k as f32) / 40.0,
            cur_time,
            d_pos,
            false,
            None,
        );
    }
}

/// Raven `void G2API_ResetRagDoll(CGhoul2Info_v &ghoul2)` -> `G2_ResetRagDoll`
/// (`G2_bones.cpp:1573`): re-inits the bone list (`G2_Init_Bone_List`) and
/// clears the `GHOUL2_RAG_PENDING|GHOUL2_RAG_DONE|GHOUL2_RAG_STARTED` flags on
/// the first model with a valid `mModelindex`, a no-op if none is ragging.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:1409-1413`, `G2_bones.cpp:1573-1620`
pub fn g2api_reset_ragdoll(g2: &mut Ghoul2System, ghoul2: &mut CGhoul2Info_v) {
    let count = ghoul2.size(g2);
    let mut model = 0;
    while model < count {
        if ghoul2.get(g2, model).modelindex != -1 {
            break;
        }
        model += 1;
    }
    if model == count {
        return;
    }
    let info = ghoul2.get_mut(g2, model);
    if info.flags & GHOUL2_RAG_STARTED == 0 {
        // no use in doing anything if we aren't ragging
        return;
    }
    bones::g2_init_bone_list(&mut info.blist);
    info.flags &= !(GHOUL2_RAG_PENDING | GHOUL2_RAG_DONE | GHOUL2_RAG_STARTED);
}

/// Raven `void G2API_AnimateG2Models(CGhoul2Info_v &ghoul2, int AcurrentTime,
/// CRagDollUpdateParams *params)` (the ragdoll overload, disambiguated from
/// the non-rag `G2API_AnimateG2Models(CGhoul2Info_v&, float speedVar)` owned
/// by `api_bones.rs`) — walks every model with `mModel` set and calls
/// `G2_Animate_Bone_List` (`bones.rs`) per model.
///
/// `CRagDollUpdateParams` (`G2_gore.h:94`) is the §F17 `RagDollUpdateParams`
/// enum (`G2SV-D8`, `ragdoll_update_params.rs`); MP instantiates only the
/// no-op `Server` base (`sv_game.cpp:1539`).
///
/// Frozen verbatim in `docs/subsystems/ghoul2-server.md` `## Seam definition`.
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:1434-1465`
pub fn g2api_animate_g2_models_rag(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghoul2: &mut CGhoul2Info_v,
    a_current_time: i32,
    params: &mut RagDollUpdateParams,
) {
    let current_time = api_collision::g2api_get_time(g2, a_current_time);
    let count = ghoul2.size(g2);
    let mut model = 0;
    while model < count {
        if ghoul2.get(g2, model).model != 0 {
            bones::g2_animate_bone_list(g2, host, ghoul2, current_time, model, Some(&mut *params));
        }
        model += 1;
    }
}

/// Raven `qboolean G2API_RagPCJConstraint(CGhoul2Info_v &ghoul2, const char
/// *boneName, vec3_t min, vec3_t max)` — writes `min`/`max` into the rag
/// bone's `minAngles`/`maxAngles`; `qfalse` if the bone isn't found/ragging
/// (`G2_GetRagBoneConveniently`) or isn't `RAG_PCJ`-flagged.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:1499-1517`
pub fn g2api_rag_pcj_constraint(
    g2: &mut Ghoul2System,
    ghoul2: &mut CGhoul2Info_v,
    bone_name: &str,
    min: vec3_t,
    max: vec3_t,
) -> bool {
    let Some(idx) = find_rag_bone_index(g2, ghoul2, bone_name) else {
        return false;
    };
    let info = ghoul2.get_mut(g2, 0);
    let bone = &mut info.blist[idx];
    if bone.RagFlags & RAG_PCJ == 0 {
        return false;
    }
    bone.minAngles = min;
    bone.maxAngles = max;
    true
}

/// Raven `qboolean G2API_RagPCJGradientSpeed(CGhoul2Info_v &ghoul2, const char
/// *boneName, const float speed)` — writes `speed` into the rag bone's
/// `overGradSpeed`; `qfalse` if the bone isn't found/ragging or isn't
/// `RAG_PCJ`-flagged.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:1519-1536`
pub fn g2api_rag_pcj_gradient_speed(
    g2: &mut Ghoul2System,
    ghoul2: &mut CGhoul2Info_v,
    bone_name: &str,
    speed: f32,
) -> bool {
    let Some(idx) = find_rag_bone_index(g2, ghoul2, bone_name) else {
        return false;
    };
    let info = ghoul2.get_mut(g2, 0);
    let bone = &mut info.blist[idx];
    if bone.RagFlags & RAG_PCJ == 0 {
        return false;
    }
    bone.overGradSpeed = speed;
    true
}

/// Raven `qboolean G2API_RagEffectorGoal(CGhoul2Info_v &ghoul2, const char
/// *boneName, vec3_t pos)` — `pos` is a nullable in-param (`if (!pos)`,
/// `:1552`): `None` clears `bone->hasOverGoal`, `Some(pos)` copies into
/// `overGoalSpot` and sets it. `qfalse` if the bone isn't found/ragging or
/// isn't `RAG_EFFECTOR`-flagged.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:1538-1562`
pub fn g2api_rag_effector_goal(
    g2: &mut Ghoul2System,
    ghoul2: &mut CGhoul2Info_v,
    bone_name: &str,
    pos: Option<vec3_t>,
) -> bool {
    let Some(idx) = find_rag_bone_index(g2, ghoul2, bone_name) else {
        return false;
    };
    let info = ghoul2.get_mut(g2, 0);
    let bone = &mut info.blist[idx];
    if bone.RagFlags & RAG_EFFECTOR == 0 {
        return false;
    }
    match pos {
        None => bone.hasOverGoal = false,
        Some(pos) => {
            bone.overGoalSpot = pos;
            bone.hasOverGoal = true;
        }
    }
    true
}

/// Raven `qboolean G2API_GetRagBonePos(CGhoul2Info_v &ghoul2, const char
/// *boneName, vec3_t pos, vec3_t entAngles, vec3_t entPos, vec3_t entScale)`
/// — the entire body is `{ return qfalse; }` ("do something?"): `pos` is
/// never written on the sole observed path, so per the frozen out-param
/// discriminator (`G2SV-D1` generalized, the doc's own cited archetype
/// alongside `GetBoneAnim`/`GetAnimRange`) this maps to `Option<vec3_t>`
/// (`None` = the untouched-output path), not a write-through `&mut`
/// out-param. `entAngles`/`entPos`/`entScale` are the entity transform
/// inputs the (never-implemented) bone-position computation would need.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:1564-1567`
pub fn g2api_get_rag_bone_pos(
    _g2: &mut Ghoul2System,
    _ghoul2: &mut CGhoul2Info_v,
    _bone_name: &str,
    _ent_angles: vec3_t,
    _ent_pos: vec3_t,
    _ent_scale: vec3_t,
) -> Option<vec3_t> {
    None
}

/// Raven `qboolean G2API_RagEffectorKick(CGhoul2Info_v &ghoul2, const char
/// *boneName, vec3_t velocity)` — zeroes the bone's Z `epVelocity`, adds
/// `velocity` in, and clears `physicsSettled`; `qfalse` if the bone isn't
/// found/ragging or isn't `RAG_EFFECTOR`-flagged.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:1569-1588`
pub fn g2api_rag_effector_kick(
    g2: &mut Ghoul2System,
    ghoul2: &mut CGhoul2Info_v,
    bone_name: &str,
    velocity: vec3_t,
) -> bool {
    let Some(idx) = find_rag_bone_index(g2, ghoul2, bone_name) else {
        return false;
    };
    let info = ghoul2.get_mut(g2, 0);
    let bone = &mut info.blist[idx];
    if bone.RagFlags & RAG_EFFECTOR == 0 {
        return false;
    }
    bone.epVelocity[2] = 0.0;
    bone.epVelocity[0] += velocity[0];
    bone.epVelocity[1] += velocity[1];
    bone.epVelocity[2] += velocity[2];
    bone.physicsSettled = false;
    true
}

/// Raven `qboolean G2API_RagForceSolve(CGhoul2Info_v &ghoul2, qboolean
/// force)` — set/clear `GHOUL2_RAG_FORCESOLVE` on `ghoul2[0].mFlags`;
/// `qfalse` if not currently ragging (`GHOUL2_RAG_STARTED` unset).
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:1590-1610`
pub fn g2api_rag_force_solve(
    g2: &mut Ghoul2System,
    ghoul2: &mut CGhoul2Info_v,
    force: bool,
) -> bool {
    let info = ghoul2.get_mut(g2, 0);
    if info.flags & GHOUL2_RAG_STARTED == 0 {
        return false;
    }
    if force {
        info.flags |= GHOUL2_RAG_FORCESOLVE;
    } else {
        info.flags &= !GHOUL2_RAG_FORCESOLVE;
    }
    true
}

/// Raven `qboolean G2API_SetBoneIKState(CGhoul2Info_v &ghoul2, int time,
/// const char *boneName, int ikState, sharedSetBoneIKStateParams_t *params)`
/// -> `G2_SetBoneIKState` (`G2_bones.cpp:4663`). `boneName` is a nullable
/// in-param (`if (!boneName)`, `:4674`: null means init/reset IK state on
/// every bone); `params` is required except when `ikState == IKS_NONE`
/// (`:4696-4701`). `ikState` stays the raw `int` the Raven signature
/// declares (not the `sharedEIKMoveState` enum type) for exact 1:1 arity
/// (`G2SV-D6`). Reads the model's `mdxaHeader_t` via `G2_GetModA`
/// (`EngineHost::model_mdxa`) and reaches `G2_ConstructGhoulSkeleton`/
/// `G2_RagDollSetup` (both host-consuming, `ragdoll.rs`/`render/skeleton.rs`).
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:1613-1616`, `G2_bones.cpp:4663-4813`
#[allow(clippy::too_many_arguments)]
pub fn g2api_set_bone_ik_state(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghoul2: &mut CGhoul2Info_v,
    time: i32,
    bone_name: Option<&str>,
    ik_state: i32,
    params: Option<&mut sharedSetBoneIKStateParams_t>,
) -> bool {
    let model = 0;
    let cur_time = time;
    let rmod_a = g2_get_mod_a(g2, ghoul2.get(g2, model));

    let Some(bone_name) = bone_name else {
        // null bonename param means it's time to init the ik stuff on this instance
        if ik_state == sharedEIKMoveState::IKS_NONE as i32 {
            // this means we want to reset the IK state completely
            let info = ghoul2.get_mut(g2, model);
            for bone in info.blist.iter_mut() {
                if bone.boneNumber != -1 {
                    bone.flags &= !BONE_ANGLES_RAGDOLL;
                    bone.flags &= !BONE_ANGLES_IK;
                    bone.RagFlags = 0;
                    bone.lastTimeUpdated = 0;
                }
            }
            return true;
        }
        let Some(params) = params else {
            return false;
        };
        let s_rdup = sharedRagDollUpdateParams_t {
            angles: params.angles,
            position: params.origin,
            scale: params.scale,
            velocity: [0.0, 0.0, 0.0],
            me: 0,
            settle_frame: 0,
        };
        g2_init_ik(g2, host, ghoul2, &s_rdup, cur_time, model);
        return true;
    };

    if rmod_a.is_none() {
        return false;
    }
    // `mod_a` (Raven `model_t *`) is the raw anim-model pointer; the port
    // never names `model_t` (G2SV-D5) — `ghoul2.animModel` already IS that
    // pointer's port-time shape, an `Option<MdxaView>` (`shared/cghoul2_info.rs`).
    let anim_model = ghoul2.get(g2, model).anim_model;
    if anim_model.is_none() {
        return false;
    }

    let mut index = bones::g2_find_bone(anim_model, &ghoul2.get(g2, model).blist, bone_name);
    if index == -1 {
        let info = ghoul2.get_mut(g2, model);
        index = bones::g2_add_bone(anim_model, &mut info.blist, bone_name);
    }
    if index == -1 {
        // couldn't find or add the bone
        return false;
    }
    let idx = index as usize;

    if ik_state == sharedEIKMoveState::IKS_NONE as i32 {
        let info = ghoul2.get_mut(g2, model);
        let bone = &mut info.blist[idx];
        if bone.flags & BONE_ANGLES_RAGDOLL == 0 {
            // you can't set the ik state to none if it's not a rag/ik bone
            return false;
        }
        // keep it on the rag list, remove it as an IK bone instead
        bone.flags &= !BONE_ANGLES_RAGDOLL;
        bone.flags |= BONE_ANGLES_IK;
        bone.RagFlags &= !RAG_PCJ_IK_CONTROLLED;
        return true;
    }

    // need params if we're not resetting
    let Some(params) = params else {
        // Raven: `assert(0); return qfalse;`
        return false;
    };

    if ghoul2.get(g2, model).blist[idx].flags & BONE_ANGLES_RAGDOLL != 0 {
        // already flagged as rag, can't set it again
        return false;
    }

    let (world_matrix, _world_matrix_inv) =
        misc::g2_generate_world_matrix(params.angles, params.origin);
    skeleton::g2_construct_ghoul_skeleton(g2, host, ghoul2, cur_time, false, params.scale);

    let mut pcj_flags = RAG_PCJ | RAG_PCJ_IK_CONTROLLED | RAG_PCJ_POST_MULT | RAG_EFFECTOR;
    if params.pcj_overrides != 0 {
        pcj_flags = params.pcj_overrides;
    }

    {
        let info = ghoul2.get_mut(g2, model);
        let bone = &mut info.blist[idx];
        bone.ikSpeed = 0.4;
        bone.ikPosition = [0.0, 0.0, 0.0];
    }

    g2_set_bone_rag(
        g2,
        ghoul2,
        model,
        &world_matrix,
        bone_name,
        params.scale,
        params.origin,
    );

    let start_frame = params.start_frame;
    let end_frame = params.end_frame;
    let needs_anim = {
        let info = ghoul2.get(g2, model);
        let bone = &info.blist[idx];
        bone.startFrame != start_frame
            || bone.endFrame != end_frame
            || params.force_anim_on_bone != 0
    };
    if needs_anim {
        let info = ghoul2.get_mut(g2, model);
        g2_set_bone_anim_no_bs(
            info,
            bone_name,
            start_frame,
            end_frame - 1,
            BONE_ANIM_OVERRIDE_FREEZE | BONE_ANIM_BLEND,
            1.0,
        );
    }

    skeleton::g2_construct_ghoul_skeleton(g2, host, ghoul2, cur_time, false, params.scale);

    ghoul2.get_mut(g2, model).blist[idx].lastTimeUpdated = 0;

    {
        let pcj_mins = params.pcj_mins;
        let pcj_maxs = params.pcj_maxs;
        let blend_time = params.blend_time;
        let radius = params.radius;
        let info = ghoul2.get_mut(g2, model);
        g2_set_bone_angles_rag(
            host,
            info,
            bone_name,
            pcj_flags,
            radius,
            Some(pcj_mins),
            Some(pcj_maxs),
            blend_time,
            cur_time,
        );
    }

    if !call_rag_doll_setup(
        g2,
        host,
        ghoul2,
        model,
        cur_time,
        true,
        params.origin,
        false,
    ) {
        // Raven: `assert(!"failed to add any rag bones"); return qfalse;`
        return false;
    }
    true
}

/// Raven `qboolean G2API_IKMove(CGhoul2Info_v &ghoul2, int time,
/// sharedIKMoveParams_t *params)` -> `G2_IKMove` (`G2_bones.cpp:4816`); the
/// live (non-`#if 0`) arm dereferences `params` unconditionally (no null
/// check on that path) and calls `G2_RagDollSetup` (host-consuming,
/// `ragdoll.rs`) before writing `params->desiredOrigin`/`movementSpeed` onto
/// every current rag bone.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:1618-1622`, `G2_bones.cpp:4816-4876`
pub fn g2api_ik_move(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghoul2: &mut CGhoul2Info_v,
    time: i32,
    params: &mut sharedIKMoveParams_t,
) -> bool {
    let model = 0;
    if !call_rag_doll_setup(g2, host, ghoul2, model, time, true, params.origin, false) {
        // changed models, possibly
        return false;
    }

    let info_ptr = split_info(g2, ghoul2, model);
    // SAFETY: see `split_info`'s module-doc note.
    let info = unsafe { &mut *info_ptr };
    let num_rags = g2.rag.num_rags;
    for i in 0..num_rags {
        if let Some(bone) = info
            .blist
            .iter_mut()
            .find(|b| b.boneNumber >= 0 && b.ragIndex == i)
        {
            bone.ikPosition = params.desired_origin;
            bone.ikSpeed = params.movement_speed;
        }
    }
    true
}

/// Raven `static inline boneInfo_t *G2_GetRagBoneConveniently(CGhoul2Info_v
/// &ghoul2, const char *boneName)` — resolves `ghoul2[0]`'s named bone,
/// `None` if it isn't found or the model isn't ragging
/// (`GHOUL2_RAG_STARTED` unset); the `RAG_PCJ`/`RAG_EFFECTOR` flag check the
/// four `G2API_Rag*` callers each add is left to them (their own guard
/// differs per caller). Returns an index rather than a borrowed
/// `&mut boneInfo_t` so callers can re-derive their own mutable borrow
/// afterward without holding this fn's borrow alive.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:1465-1481`
fn find_rag_bone_index(
    g2: &Ghoul2System,
    ghoul2: &CGhoul2Info_v,
    bone_name: &str,
) -> Option<usize> {
    let info = ghoul2.get(g2, 0);
    if info.flags & GHOUL2_RAG_STARTED == 0 {
        return None;
    }
    let bone_index = g2_find_bone_rag(info, bone_name);
    if bone_index < 0 {
        return None;
    }
    let bone = &info.blist[bone_index as usize];
    if bone.flags & BONE_ANGLES_RAGDOLL == 0 {
        return None;
    }
    Some(bone_index as usize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::ffi::c_void;
    use mp_host_interface::mdx::mdxa::{MdxaParsed, MdxaView};
    use mp_host_interface::mock::MockHost;

    /// `skel_bone_name_matches`'s raw byte arithmetic is the trickiest part
    /// of this file (`G2SV-D5` forbids naming `mdxaHeader_t`/`mdxaSkel_t`, so
    /// it replicates the oracle's wire offsets unchanged) — exercise it
    /// against a synthetic in-memory `.gla`-shaped buffer.
    #[test]
    fn skel_bone_name_matches_reads_wire_layout() {
        // `sizeof(mdxaHeader_t)` == 100, `ofsEnd` @96 (`mdx_format.h:351-371`).
        const MDXA_HEADER_SIZE: usize = 100;
        const OFS_END: usize = 96;
        const SKEL_SIZE: usize = 176;
        // header (100 bytes) + a 2-entry offset table + two skeleton entries
        // whose `name` field (offset 0) is all this reads.
        let mut buf = vec![0u8; MDXA_HEADER_SIZE + 8 + SKEL_SIZE * 2];
        let ofs_end = buf.len() as i32;
        buf[OFS_END..OFS_END + 4].copy_from_slice(&ofs_end.to_le_bytes());
        // offsets[0] = 8 (right after the 2-entry offset table itself).
        buf[MDXA_HEADER_SIZE..MDXA_HEADER_SIZE + 4].copy_from_slice(&8i32.to_le_bytes());
        // offsets[1] = 8 + SKEL_SIZE.
        buf[MDXA_HEADER_SIZE + 4..MDXA_HEADER_SIZE + 8]
            .copy_from_slice(&((8 + SKEL_SIZE) as i32).to_le_bytes());
        let skel0 = MDXA_HEADER_SIZE + 8;
        let skel1 = MDXA_HEADER_SIZE + 8 + SKEL_SIZE;
        buf[skel0..skel0 + 11].copy_from_slice(b"model_root\0");
        buf[skel1..skel1 + 7].copy_from_slice(b"pelvis\0");
        // numBones @84 — the parse-once sidecar sizes its skel table off it.
        buf[84..88].copy_from_slice(&2i32.to_le_bytes());

        let view = unsafe { MdxaView::from_block(buf.as_ptr() as *const c_void) };
        let parsed: &'static MdxaParsed = Box::leak(Box::new(MdxaParsed::parse(view)));
        let header = Some(MdxaRef { parsed, view });
        assert!(skel_bone_name_matches(header, 0, "MODEL_ROOT")); // stricmp: case-insensitive
        assert!(skel_bone_name_matches(header, 1, "pelvis"));
        assert!(!skel_bone_name_matches(header, 0, "pelvis"));
        assert!(!skel_bone_name_matches(header, 1, "model_root"));
        assert!(!skel_bone_name_matches(None, 0, "model_root"));
    }

    /// [`rag_random_angle_seed`] transcription check against the deterministic
    /// `holdrand`-backed `MockHost` (a fully-built fixture, `G2SV-D14`): it must
    /// consume exactly three `flrand` draws per axis, in order, and apply the
    /// oracle lerp `(min-max)*scalar + max` bit-for-bit.
    ///
    /// Note: the retail-i686 "heavily-central, stays inside `[min, max]`"
    /// property does NOT hold on this LP64 host. Under the c_ulong `holdrand`
    /// ruling (2026-07-09, reversing the earlier u32 normalization; commit
    /// 0b41dc4e), `flrand`'s `(float)(holdrand >> 17)` pulls the full
    /// platform-width state — not the 32-bit "0-32767 range" the oracle
    /// comment assumes under `unsigned long` wraparound — so after a few LCG
    /// steps a draw exceeds `[-1, 1)` and the seed leaves `[min, max]`. The
    /// oracle on the same LP64 referee produces the identical out-of-bounds
    /// seed, so this reproduces the ruled behavior rather than the stale
    /// in-bounds invariant; it pins the draw count/order and the lerp against
    /// a second identically-seeded host.
    #[test]
    fn rag_random_angle_seed_reproduces_lerp_stream() {
        let mut host = MockHost::new();
        host.rand_init(1);
        // Independent host on the same seed reproduces the exact flrand stream
        // the function must consume.
        let mut reference = MockHost::new();
        reference.rand_init(1);

        let min = [-90.0, -45.0, -45.0];
        let max = [90.0, 45.0, 45.0];
        for _ in 0..16 {
            let angles = rag_random_angle_seed(&mut host, min, max);
            for k in 0..3 {
                let mut scalar = reference.flrand(-1.0, 1.0);
                scalar *= reference.flrand(-1.0, 1.0) * reference.flrand(-1.0, 1.0);
                scalar *= 0.5;
                scalar += 0.5;
                let expected = (min[k] - max[k]) * scalar + max[k];
                assert_eq!(
                    angles[k], expected,
                    "angles[{k}]={} != reproduced lerp {expected}",
                    angles[k]
                );
            }
        }
    }
}
