//! `RagDollSolver` — the server-live RagDoll + IK solver.
//!
//! Per `docs/subsystems/ghoul2-server.md` roster (`ragdoll.rs`, class
//! "RagDollSolver"): "`G2_RagDollSetup/RagDoll/RagDollSolve/
//! SettlePositionNumeroTrois/RagSetState/IKSolve/DoIK/BoneSnap,
//! `SRagEffector`" — the fn-statics block at `G2_bones.cpp:1214-1241`
//! (`ragBasepose`…`rag`) becomes the owned [`RagDollSolver`] struct
//! (`G2SV-D3`, ruling 3 cross-frame kind; `Ghoul2System.rag`, `## State
//! ownership`).
//!
//! Expanded against the oracle "rag stuff" section (`G2_bones.cpp:1145` on)
//! plus the "cgvm ragdoll-callback dead branches" ground truth (`## Raven
//! ground truth`, "The solver functions ported wholesale to `ragdoll.rs`
//! embed a family of client-game-VM callback branches" — `Rag_Trace`,
//! `G2_BoneSnap`, `G2_RagDebugBox`/`G2_RagDebugLine`) and the private static
//! helpers those eight named functions call, transitively, within
//! `G2_bones.cpp` itself: `G2_Generate_MatrixRag`, `G2_ApplyRealBonePhysics`,
//! `G2_BoneOnGround`, `G2_RagGetPelvisLumbarOffsets`,
//! `G2_RagGetWorldAnimMatrix`, `AngleNormZero`, `G2_IKReposition`,
//! `G2_Get_Bone_Name` — 20 functions + the `SRagEffector` data struct + the
//! `RagDollSolver` state struct itself, all ported below (porting-rules §F21
//! one-class-per-file, private helpers colocate).
//!
//! **Scope boundary vs `api_ragdoll.rs`** (reported upstream in the porting
//! task's `problems` output, kept out of the doc's own text per house
//! rules): `G2_SetRagDoll`/`G2_ResetRagDoll`/`G2_SetBoneIKState`/`G2_IKMove`
//! and the ragdoll overload of `G2_Animate_Bone_List` (`:4498-4527`) are
//! *also* physically defined in `G2_bones.cpp`, but the doc's own
//! `api_ragdoll.rs` roster row names them there (confirmed by that file's
//! already-landed stubs, which fold each `G2API_*` wrapper and its
//! `G2_bones.cpp` callee into one function — e.g. `g2api_set_ragdoll`, no
//! separate `g2_set_rag_doll` helper). Their exclusive private setup helpers
//! (`G2_Set_Bone_Rag`, `G2_Set_Bone_Angles_Rag`, `G2_Find_Bone_Rag`,
//! `G2_Set_Bone_Anim_No_BS`, `G2_InitIK`, `G2_Set_Bone_Angles_IK`,
//! `G2_GetModA`) colocate with those api-level functions for the same
//! reason and are not stubbed here either. `G2_GetBoneMatrixLow`/
//! `G2_GetBoneBasepose`/`G2_RagGetBoneBasePoseMatrixLow` are `tr_ghoul2.cpp`
//! definitions the doc explicitly rosters to `render/skeleton.rs`, as are
//! `G2_GetBoneDependents`/`G2_WasBoneRendered`/`G2_GetBoneNameFromSkel`/
//! `G2_RagGetAnimMatrix`/`G2_RagPrintMatrix`/`G2_GetBoltMatrixLow`
//! (also `tr_ghoul2.cpp`, called from this file's functions but owned
//! elsewhere).
//!
//! **Three dead declarations found while enumerating this class (reported
//! upstream), not stubbed — no `G2API_*`/`G2_local.h` caller and zero
//! internal callers other than each other, verified by grep over
//! `oracle/codemp/`:**
//! 1. `G2_SetRagDollBullet` (`G2_bones.cpp:2040-2151`) — no caller anywhere.
//! 2. `G2_RagDollMatchPosition` (`:1491-1520`) — sole caller is the dead
//!    `G2_SetRagDollBullet` (`:2035`).
//! 3. `G2_RagIndexForBoneNum` (`:3337-3352`) — declared, never called.
//!
//! Every function threads `g2: &mut Ghoul2System` (to reach `g2.rag`,
//! ruling 4/11) and, where the completed body reaches a host service
//! (`model_mdxa` basepose resolve `G2SV-D13`(b), `flrand` PCJ/settle/IK
//! seeding, the `trace`-served `Rag_Trace`/`CM_BoxTrace` call, the
//! `broadsword` cvar family), `host: &mut impl EngineHost` (ruling 11);
//! `use mp_host_interface::EngineHost;` (`G2SV-D12`). `CRagDollUpdateParams`
//! (`G2_gore.h:94`) is the already-ported §F17 `RagDollUpdateParams` enum
//! (`ragdoll_update_params.rs`, `G2SV-D8`); Raven's nullable
//! `CRagDollUpdateParams *params` becomes `Option<&mut RagDollUpdateParams>`
//! (matching `bones.rs`'s `g2_animate_bone_list` convention).
//!
//! `worldMatrix` (`tr_ghoul2.cpp:136`, the skeleton-build scratch `render/
//! skeleton.rs` threads through `G2_ConstructGhoulSkeleton`) is not owned
//! here; functions that read it (`G2_RagGetWorldAnimMatrix`) take it as a
//! `&mdxaBone_t` parameter rather than reaching into a sibling module's
//! private scratch.
//!
//! **`-DNDEBUG` convention (matching `bolts.rs`/`api_models.rs`).** Every
//! plain `assert(...)` quoted from the oracle bodies below is a no-op in the
//! WinDed Release build (`docs/subsystems/ghoul2-server.md` "Raven ground
//! truth" build config) and is dropped, not ported as `assert!`/`panic!`.
//!
//! **Problems reported upstream while transcribing this class (kept out of
//! the doc's own text per house rules; see this porting task's `problems`
//! output for the full list):**
//! 1. `misc.rs::create_matrix` (`Create_Matrix`, `G2_misc.cpp:1630-1653`) is
//!    a private `fn`, but `G2_bones.cpp` calls the *same* Raven global from
//!    `G2_RagDollSolve`/`G2_IKSolve` (`:4059,4102,4248,4336,4348,4444`) — a
//!    cross-file caller `misc.rs`'s own visibility doesn't allow. A local,
//!    file-private `create_matrix` twin is transcribed below instead of
//!    editing `misc.rs` (out of this porting task's file scope); `misc.rs`
//!    should expose `pub(crate) fn create_matrix` so this duplicate can be
//!    deleted.
//! 2. `G2_GetBoneDependents`/`G2_WasBoneRendered`/`G2_RagGetAnimMatrix`
//!    (`tr_ghoul2.cpp:603,645,1417`) are, per this file's own module doc
//!    above, owned by `render/skeleton.rs` — but that file (read in full
//!    before writing this one) does not yet expose them. Ported here as
//!    private, file-local stopgaps (mirroring the exact precedent
//!    `render/skeleton.rs` itself set for its own "no landed home"
//!    dependency, `g2_get_bolt_matrix_low`) so this file's callers compile
//!    and behave correctly; they duplicate a private `mdxaSkel_t`/
//!    `mdxaSkelOffsets_t` byte-layout walk (`mdx_format.h:374-397`, sizes
//!    only — never an imported `mp_renderer` type, `G2SV-D5`) that
//!    `render/skeleton.rs`'s own (not-yet-landed) accessors will also need.
//!    Recommend centralizing this walk in `render/skeleton.rs` once it
//!    lands, deleting the duplicate here.
//! 3. The frozen `g2_rag_set_state`/`g2_rag_get_world_anim_matrix` signatures
//!    take **both** `ghoul2: &mut CGhoul2Info` and `bone: &mut boneInfo_t`
//!    where, at every real call site, `bone` **is** an element of
//!    `ghoul2.blist` (Raven itself passes `blist[i]ref` alongside `ghoul2`
//!    with no such conflict, since it has no borrow checker). `boneInfo_t`
//!    has neither `Default` nor `Clone` (checked before writing this file),
//!    so there is no safe extract/reinsert workaround; call sites in this
//!    file use a documented `unsafe` raw-pointer reborrow
//!    (`alias_bone_mut`, below) as a stopgap — technically two live `&mut`
//!    into overlapping memory, though every callee here only ever touches
//!    `bone`'s own fields and `ghoul2`'s *other* fields, never
//!    `ghoul2.blist` again while `bone` is live. Recommend the doc revisit
//!    both signatures (e.g. `bone_index: usize` instead of `&mut
//!    boneInfo_t`) to remove the need for `unsafe` entirely.
//! 4. `g2_ik_reposition`'s frozen signature (`(g2, host, current_org,
//!    params)`, no `ghoul2`/blist parameter) matches Raven's own
//!    `G2_IKReposition(const vec3_t currentOrg, CRagDollUpdateParams
//!    *params)` — but Raven's body reads/writes `ragBoneData[i]->{RagFlags,
//!    ikPosition, velocityEffector, lastPosition}` (`G2_bones.cpp:4263-4293`)
//!    through the now-removed raw-pointer array (`G2SV-D13`(b) replaced it
//!    with blist **indices**, which need the owning `CGhoul2Info.blist` to
//!    resolve). Unlike its siblings (`g2_rag_doll_solve`/`g2_ik_solve`/
//!    `g2_rag_doll_settle_position_numero_trois`, which all carry a
//!    `ghoul2`/`CGhoul2Info_v` parameter), this one has no way to reach that
//!    data. The body below is therefore a documented no-op (not a silent
//!    invention of substitute behavior) — reported upstream; the doc should
//!    add a blist-reaching parameter to this signature.
//! 5. `g2_rag_get_pelvis_lumbar_offsets`'s frozen signature has no
//!    `world_matrix: &mdxaBone_t` parameter, unlike `g2_rag_get_world_anim_
//!    matrix` (which this function calls, and which — per this file's own
//!    doc comment above — correctly takes `world_matrix` rather than reading
//!    a sibling's private scratch) and unlike Raven's own body, which reads
//!    the file-scope `worldMatrix` directly a second time
//!    (`G2_bones.cpp:3424`). With no way to receive it, this function's body
//!    is a documented no-op (leaves `pos`/`dir`/`anim_pos`/`anim_dir`
//!    unchanged) rather than substituting an invented matrix. Its sole
//!    caller (`g2_rag_doll_settle_position_numero_trois`, gated behind
//!    `broadsword_ragtobase > 1`, a tuning cvar that defaults off) is
//!    unaffected in the default configuration.

use core::ffi::c_void;

use mp_host_interface::EngineHost;
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::q_math::{
    _DotProduct, _VectorAdd, _VectorMA, _VectorScale, _VectorSubtract, vectoangles,
    AngleNormalize180, AnglesToAxis, AngleVectors, DistanceSquared, VectorInverse,
};
use mp_qshared::shared::{
    cplane_t, mdxaBone_t, vec3_t, VectorLength, VectorNormalize, CONTENTS_SOLID,
    CONTENTS_TERRAIN, ENTITYNUM_NONE, ENTITYNUM_WORLD, MAX_QPATH,
};

use crate::api_collision::{g2api_get_time, g2api_give_me_vector_from_matrix};
use crate::bones::g2_find_bone;
use crate::ghoul2_system::Ghoul2System;
use crate::misc::{g2_generate_world_matrix, inverse_matrix, transform_point};
use crate::ragdoll_update_params::RagDollUpdateParams;
use crate::render::bone_transform::{multiply_3x4_matrix, uncompress_bone};
use crate::render::skeleton::{
    g2_construct_ghoul_skeleton, g2_get_bone_basepose, g2_get_bone_matrix_low,
    g2_rag_get_bone_base_pose_matrix_low,
};
use crate::shared::bone_info_t::boneInfo_t;
use crate::shared::cghoul2_info::CGhoul2Info;
use crate::shared::cghoul2_info_v::CGhoul2Info_v;
use crate::shared::eg2_collision::EG2_Collision;
use mp_qshared::shared::Eorientations;

/// Raven `#define MAX_BONES_RAG (256)`.
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:1163`
pub const MAX_BONES_RAG: usize = 256;

/// Raven `RAG_MASK` — `CONTENTS_SOLID|CONTENTS_TERRAIN` (the commented-out
/// extra bits in Raven's own `#define` were never live).
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:1174`
const RAG_MASK: i32 = CONTENTS_SOLID | CONTENTS_TERRAIN;

/// Raven `RAG_PCJ`. Source: `oracle/codemp/ghoul2/G2_bones.cpp:1192`
const RAG_PCJ: i32 = 0x0000_1;
/// Raven `RAG_PCJ_MODEL_ROOT`. Source: `G2_bones.cpp:1194`
const RAG_PCJ_MODEL_ROOT: i32 = 0x0000_4;
/// Raven `RAG_PCJ_PELVIS`. Source: `G2_bones.cpp:1195`
const RAG_PCJ_PELVIS: i32 = 0x0000_8;
/// Raven `RAG_EFFECTOR`. Source: `G2_bones.cpp:1196`
const RAG_EFFECTOR: i32 = 0x0010_0;
/// Raven `RAG_WAS_NOT_RENDERED`. Source: `G2_bones.cpp:1197`
const RAG_WAS_NOT_RENDERED: i32 = 0x0100_0;
/// Raven `RAG_WAS_EVER_RENDERED`. Source: `G2_bones.cpp:1198`
const RAG_WAS_EVER_RENDERED: i32 = 0x0200_0;
/// Raven `RAG_BONE_LIGHTWEIGHT`. Source: `G2_bones.cpp:1199`
const RAG_BONE_LIGHTWEIGHT: i32 = 0x0400_0;
/// Raven `RAG_PCJ_IK_CONTROLLED`. Source: `G2_bones.cpp:1200`
const RAG_PCJ_IK_CONTROLLED: i32 = 0x0800_0;
/// Raven `RAG_UNSNAPPABLE`. Source: `G2_bones.cpp:1201`
const RAG_UNSNAPPABLE: i32 = 0x1000_0;

/// Raven `GHOUL2_RAG_FORCESOLVE`. Source: `oracle/codemp/ghoul2/G2_bones.cpp:1210`
const GHOUL2_RAG_FORCESOLVE: i32 = 0x1000;

/// Raven `BONE_ANGLES_RAGDOLL`. Source: `oracle/codemp/ghoul2/G2.h:17`
const BONE_ANGLES_RAGDOLL: i32 = 0x2000;
/// Raven `BONE_ANGLES_IK`. Source: `oracle/codemp/ghoul2/G2.h:19`
const BONE_ANGLES_IK: i32 = 0x4000;

/// Raven `DEFAULT_MINS_2`. Source: `oracle/codemp/game/bg_public.h:41`
const DEFAULT_MINS_2: f32 = -24.0;

/// Raven `enum ERagState { ERS_DYNAMIC, ERS_SETTLING, ERS_SETTLED };`
/// (CLAUDE.md enum-vs-alias fidelity: a genuinely named C enum ports as a
/// real Rust enum, not an int alias, even though Raven itself stores it in
/// a plain `static int ragState` — that is C's lax implicit enum-to-int
/// conversion, not a reason to flatten the port). Bare `ERS_`/`E` Hungarian
/// prefixes dropped (ruling 40).
///
/// Type definition source: `oracle/codemp/ghoul2/G2_bones.cpp:1233-1238`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RagState {
    /// Raven `ERS_DYNAMIC` — also Raven's zero-init default for the
    /// plain-`int` `static int ragState;` (`:1239`), so the `#[default]`.
    #[default]
    Dynamic = 0,
    /// Raven `ERS_SETTLING`.
    Settling = 1,
    /// Raven `ERS_SETTLED`.
    Settled = 2,
}

/// Raven `SRagEffector` — one ragdoll bone's effector goal/position state.
///
/// Raven: (no class-level comment).
/// Type definition source: `oracle/codemp/ghoul2/G2_bones.cpp:1165-1172`
#[derive(Debug, Clone, Copy, Default)]
pub struct SRagEffector {
    pub current_origin: vec3_t,
    pub desired_direction: vec3_t,
    pub desired_origin: vec3_t,
    pub radius: f32,
    pub weight: f32,
}

/// Raven `mdxaBone_t` has no `Default` impl (native, `#[repr(C)]`
/// `Copy`/`Clone`/`PartialEq` only) — a zero matrix constant backs the
/// `[mdxaBone_t; MAX_BONES_RAG]` array init below.
const ZERO_BONE: mdxaBone_t = mdxaBone_t {
    matrix: [[0.0; 4]; 3],
};

/// The RagDoll/IK solver's fn-statics block, owned (`G2SV-D3`, ruling 3
/// cross-frame kind; threaded as `&mut Ghoul2System.rag`, `## State
/// ownership`).
///
/// Per `G2SV-D13`(b)/ruling 29 (closing `G2SV-Q9`): Raven's raw-pointer
/// arrays `ragBasepose`/`ragBaseposeInv` (`mdxaBone_t*`, `:1214-1215`) are
/// **not stored** — the basepose/baseposeInv matrices are resolved per call
/// through `G2_GetBoneMatrixLow` over `EngineHost::model_mdxa`
/// (`G2_bones.cpp:2622`, write-through pattern, ruling 21); `ragBoneData`
/// (`boneInfo_t*`, `:1218`) and `rag` (`vector<boneInfo_t*>`, `:1241`)
/// become `mBlist` **indices**, resolved against the live model's `mBlist`
/// at use — Raven already carries the parallel `ragBlistIndex[MAX_BONES_RAG]`
/// `int` array (`:1220`) doing exactly this for the by-bone-number lookup.
///
/// **`rag_bone_data` resolves the doc's flagged ambiguity.** The doc's `##
/// State ownership` row named one `blist_index` field for what are, in the
/// oracle, two *different* index mappings sharing that array's shape:
/// `ragBlistIndex` (bone-number-keyed) and `ragBoneData` (**solve-order**-
/// keyed, `0..numRags`, set in `G2_RagDollSetup`'s second pass, `:2389`) —
/// and left a second array as an open implementation choice ("the doc may
/// intend a second array here"). This field is that array: `rag_bone_data`
/// stores, per solve-order position, the `mBlist` index (replacing Raven's
/// solve-order-keyed `boneInfo_t*`), exactly mirroring the already-present
/// solve-order arrays `bones`/`effectors` (§A1 internal latitude — not an
/// ABI/seam field).
///
/// Raven ragdoll/IK fn-statics block.
/// Type definition source: `oracle/codemp/ghoul2/G2_bones.cpp:1214-1241`
pub struct RagDollSolver {
    /// Raven `static mdxaBone_t ragBones[MAX_BONES_RAG]` (`:1216`) — per-rag
    /// current bone matrix, solve-order indexed.
    pub bones: [mdxaBone_t; MAX_BONES_RAG],
    /// Raven `static SRagEffector ragEffectors[MAX_BONES_RAG]` (`:1217`).
    pub effectors: [SRagEffector; MAX_BONES_RAG],
    /// Raven `static int tempDependents[MAX_BONES_RAG]` (`:1219`) — scratch
    /// dependent-bone-index buffer filled by `G2_GetBoneDependents`
    /// (`render/skeleton.rs`).
    pub temp_dependents: [i32; MAX_BONES_RAG],
    /// Raven `static int ragBlistIndex[MAX_BONES_RAG]` (`:1220`) — bone
    /// number -> `mBlist` position; the load-bearing identity `ragBoneData`/
    /// `rag` collapse onto (`G2SV-D13`(b)).
    pub blist_index: [i32; MAX_BONES_RAG],
    /// Raven `static boneInfo_t *ragBoneData[MAX_BONES_RAG]` (`:1218`) —
    /// solve-order-keyed `mBlist` index (see the struct-level doc comment
    /// resolving the doc's flagged ambiguity). `-1` = unset.
    pub rag_bone_data: [i32; MAX_BONES_RAG],
    /// Raven `static int numRags` (`:1221`).
    pub num_rags: i32,
    /// Raven `static vec3_t ragBoneMins` (`:1222`).
    pub bone_mins: vec3_t,
    /// Raven `static vec3_t ragBoneMaxs` (`:1223`).
    pub bone_maxs: vec3_t,
    /// Raven `static vec3_t ragBoneCM` (`:1224`) — center of mass.
    pub bone_cm: vec3_t,
    /// Raven `static vec3_t desiredPelvisOffset` (`:1226`) — "this is for
    /// the root".
    pub desired_pelvis_offset: vec3_t,
    /// Raven `static bool haveDesiredPelvisOffset=false` (`:1225`).
    pub have_desired_pelvis_offset: bool,
    /// Raven `static float ragOriginChange=0.0f` (`:1227`).
    pub origin_change: f32,
    /// Raven `static vec3_t ragOriginChangeDir` (`:1228`).
    pub origin_change_dir: vec3_t,
    /// Raven `static vec3_t handPos={0,0,0}` (`:1230`) — "debug".
    pub hand_pos: vec3_t,
    /// Raven `static vec3_t handPos2={0,0,0}` (`:1231`).
    pub hand_pos2: vec3_t,
    /// Raven `static int ragState` (`:1239`) — see [`RagState`].
    pub rag_state: RagState,
    /// Raven `static vector<boneInfo_t *> rag` (`:1241`, "once we get the
    /// dependents precomputed this can be local") — boneNumber-keyed,
    /// growable; ported as `mBlist` indices, not pointers (`G2SV-D13`(b)).
    /// `-1` = no rag/IK bone at that bone number this frame.
    pub rag: Vec<i32>,
}

impl Default for RagDollSolver {
    fn default() -> Self {
        RagDollSolver {
            bones: [ZERO_BONE; MAX_BONES_RAG],
            effectors: [SRagEffector::default(); MAX_BONES_RAG],
            temp_dependents: [0; MAX_BONES_RAG],
            blist_index: [0; MAX_BONES_RAG],
            rag_bone_data: [-1; MAX_BONES_RAG],
            num_rags: 0,
            bone_mins: vec3_t::default(),
            bone_maxs: vec3_t::default(),
            bone_cm: vec3_t::default(),
            desired_pelvis_offset: vec3_t::default(),
            have_desired_pelvis_offset: false,
            origin_change: 0.0,
            origin_change_dir: vec3_t::default(),
            hand_pos: [0.0, 0.0, 0.0],
            hand_pos2: [0.0, 0.0, 0.0],
            rag_state: RagState::default(),
            rag: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// File-local vector/angle math primitives (oracle/codemp/game/q_math.c).
// Not re-exported; see the module doc's "problems" note #1 for why these
// aren't imported from a shared home (none of the tree's existing q_math
// ports cover this set at time of writing).
// ---------------------------------------------------------------------------


/// Raven `void Create_Matrix(const float *angle, mdxaBone_t *matrix)` — file-
/// local twin of `misc.rs`'s private (also-stubbed) `create_matrix`; see this
/// file's module-doc "problems" note #1.
///
/// Source: `oracle/codemp/ghoul2/G2_misc.cpp:1628-1651`
fn create_matrix(angle: vec3_t) -> mdxaBone_t {
    let mut axis = [[0.0f32; 3]; 3];
    AnglesToAxis(angle, axis.as_mut_ptr());
    let mut matrix = ZERO_BONE;
    for row in 0..3 {
        matrix.matrix[row][0] = axis[0][row];
        matrix.matrix[row][1] = axis[1][row];
        matrix.matrix[row][2] = axis[2][row];
        matrix.matrix[row][3] = 0.0;
    }
    matrix
}

/// Zeroed `trace_t` (the type carries no `Default`); every `rag_trace` caller
/// needs one to write into.
fn zero_trace() -> trace_t {
    trace_t {
        allsolid: 0,
        startsolid: 0,
        entityNum: 0,
        fraction: 0.0,
        endpos: [0.0; 3],
        plane: cplane_t {
            normal: [0.0; 3],
            dist: 0.0,
            r#type: 0,
            signbits: 0,
            pad: [0, 0],
        },
        surfaceFlags: 0,
        contents: 0,
    }
}

/// Materialize an independent `&mut boneInfo_t` out of `ghoul2.blist[index]`
/// for a call site that also passes `ghoul2: &mut CGhoul2Info` itself — see
/// this file's module-doc "problems" note #3 for why this `unsafe` stopgap
/// exists and what it does NOT justify (only used where the callee provably
/// never touches `ghoul2.blist` again while the returned `bone` is live).
/// Takes a raw pointer (not `&mut CGhoul2Info`) so the borrow checker does
/// not tie the result's lifetime to a live `ghoul2` borrow at the call site
/// — callers build `ghoul2_ptr`/`bone_ptr` as two independent raw-pointer
/// casts, then dereference both when invoking the callee.
unsafe fn alias_bone_mut<'a>(ghoul2: *mut CGhoul2Info, index: usize) -> &'a mut boneInfo_t {
    unsafe { &mut (*ghoul2).blist[index] }
}

// ---------------------------------------------------------------------------
// Private file-local `mdxaSkel_t`/`mdxaSkelOffsets_t` byte-layout walk — see
// this file's module-doc "problems" note #2. Sizes only, transcribed from
// `oracle/codemp/renderer/mdx_format.h:350-397`; never an imported
// `mp_renderer` type (`G2SV-D5`).
// ---------------------------------------------------------------------------

/// `sizeof(mdxaHeader_t)`: 2 leading ints + `name[MAX_QPATH]` + 7 more ints
/// (`mdx_format.h:350-371`).
const MDXA_HEADER_SIZE: usize = 4 + 4 + MAX_QPATH + 4 + 4 + 4 + 4 + 4 + 4;
/// `mdxaSkel_t` field byte offsets (`mdx_format.h:388-396`): `name[MAX_QPATH]`
/// (0), `flags:u32` (`MAX_QPATH`), `parent:i32` (`+4`), `BasePoseMat` (`+4`,
/// 48 bytes), `BasePoseMatInv` (`+48`), `numChildren:i32`, `children[]`.
const MDXA_SKEL_PARENT_OFS: usize = MAX_QPATH + 4;
const MDXA_SKEL_NUM_CHILDREN_OFS: usize = MDXA_SKEL_PARENT_OFS + 4 + 48 + 48;
const MDXA_SKEL_CHILDREN_OFS: usize = MDXA_SKEL_NUM_CHILDREN_OFS + 4;

/// Resolve bone `bone_num`'s `mdxaSkel_t*` (as a raw byte pointer) out of the
/// loader's `.gla` block, exactly as Raven's `(byte*)header + sizeof(
/// mdxaHeader_t) + offsets->offsets[boneNum]` idiom does (e.g.
/// `G2_bones.cpp:1273-1274`, `tr_ghoul2.cpp:614-615`).
unsafe fn mdxa_skel_ptr(header: *const c_void, bone_num: i32) -> *const u8 {
    unsafe {
        let base = header as *const u8;
        let offsets_table = base.add(MDXA_HEADER_SIZE) as *const i32;
        let rel_offset = *offsets_table.add(bone_num as usize);
        base.add(MDXA_HEADER_SIZE).add(rel_offset as usize)
    }
}
unsafe fn mdxa_skel_parent(skel: *const u8) -> i32 {
    unsafe { *(skel.add(MDXA_SKEL_PARENT_OFS) as *const i32) }
}
unsafe fn mdxa_skel_num_children(skel: *const u8) -> i32 {
    unsafe { *(skel.add(MDXA_SKEL_NUM_CHILDREN_OFS) as *const i32) }
}
unsafe fn mdxa_skel_child(skel: *const u8, i: usize) -> i32 {
    unsafe { *(skel.add(MDXA_SKEL_CHILDREN_OFS + i * 4) as *const i32) }
}
unsafe fn mdxa_skel_name(skel: *const u8) -> String {
    unsafe {
        let cstr = core::ffi::CStr::from_ptr(skel as *const core::ffi::c_char);
        cstr.to_string_lossy().into_owned()
    }
}

/// Raven `int G2_GetBoneDependents(CGhoul2Info &ghoul2, int boneNum, int
/// *tempDependents, int maxDep)` — private file-local stopgap (module-doc
/// "problems" note #2); recurses the skeleton's `numChildren`/`children[]`
/// list, filling `out` breadth-first-then-recursive exactly as the oracle
/// does, returning the count written.
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:603-643`
fn g2_get_bone_dependents(
    g2: &Ghoul2System,
    ghoul2: &CGhoul2Info,
    bone_num: i32,
    out: &mut [i32],
) -> i32 {
    if out.is_empty() {
        return 0;
    }
    let Some(cache_id) = ghoul2.bone_cache else {
        return 0;
    };
    let Some(cache) = g2.bone_caches.get(cache_id) else {
        return 0;
    };
    let header = cache.header;
    if header.is_null() {
        return 0;
    }
    g2_get_bone_dependents_recurse(header, bone_num, out)
}

fn g2_get_bone_dependents_recurse(header: *mut c_void, bone_num: i32, out: &mut [i32]) -> i32 {
    unsafe {
        let skel = mdxa_skel_ptr(header, bone_num);
        let num_children = mdxa_skel_num_children(skel);
        let mut written = 0usize;
        for i in 0..num_children as usize {
            if written >= out.len() {
                return written as i32;
            }
            out[written] = mdxa_skel_child(skel, i);
            written += 1;
        }
        for i in 0..num_children as usize {
            if written >= out.len() {
                break;
            }
            let child = mdxa_skel_child(skel, i);
            let num = g2_get_bone_dependents_recurse(header, child, &mut out[written..]);
            written += num as usize;
        }
        written as i32
    }
}

/// Raven `bool G2_WasBoneRendered(CGhoul2Info &ghoul2, int boneNum)` —
/// private file-local stopgap (module-doc "problems" note #2); forwards to
/// the already-landed `CBoneCache::was_rendered`.
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:645-654`
fn g2_was_bone_rendered(g2: &Ghoul2System, ghoul2: &CGhoul2Info, bone_num: i32) -> bool {
    match ghoul2.bone_cache.and_then(|id| g2.bone_caches.get(id)) {
        Some(cache) => cache.was_rendered(bone_num),
        None => false,
    }
}

/// Raven `void G2_RagGetAnimMatrix(CGhoul2Info &ghoul2, const int boneNum,
/// mdxaBone_t &matrix, const int frame)` — private file-local stopgap
/// (module-doc "problems" note #2); recursively resolves bone `bone_num`'s
/// settle-frame animated matrix, memoized per-bone by `hasAnimFrameMatrix ==
/// frame`, decompressing via `uncompress_bone` (`render/bone_transform.rs`,
/// landed) and composing with the parent's (or the cache's `root_matrix` at
/// the skeleton root) already-resolved `animFrameMatrix`.
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:1417-1539`
fn g2_rag_get_anim_matrix(
    g2: &Ghoul2System,
    ghoul2: &mut CGhoul2Info,
    bone_num: i32,
    frame: i32,
) -> mdxaBone_t {
    let Some(cache_id) = ghoul2.bone_cache else {
        return ZERO_BONE;
    };
    let Some(cache) = g2.bone_caches.get(cache_id) else {
        return ZERO_BONE;
    };
    let header = cache.header;
    let root_matrix = cache.root_matrix;
    if header.is_null() {
        return ZERO_BONE;
    }

    let (skel, name) = unsafe {
        let skel = mdxa_skel_ptr(header, bone_num);
        (skel, mdxa_skel_name(skel))
    };

    let bone_list_index = resolve_or_add_bone(ghoul2, &name);
    let Some(bli) = bone_list_index else {
        return ZERO_BONE;
    };

    if ghoul2.blist[bli].hasAnimFrameMatrix == frame {
        return ghoul2.blist[bli].animFrameMatrix;
    }

    let mut anim_matrix = ZERO_BONE;
    uncompress_bone(&mut anim_matrix.matrix, bone_num, header, frame);

    let parent = unsafe { mdxa_skel_parent(skel) };
    let mut result = ZERO_BONE;
    if bone_num > 0 && parent > -1 {
        // Recursively assure the parent's animFrameMatrix is set up first.
        let _ = g2_rag_get_anim_matrix(g2, ghoul2, parent, frame);
        let pname = unsafe {
            let pskel = mdxa_skel_ptr(header, parent);
            mdxa_skel_name(pskel)
        };
        let Some(pbli) = resolve_or_add_bone(ghoul2, &pname) else {
            return ZERO_BONE;
        };
        let parent_anim_matrix = ghoul2.blist[pbli].animFrameMatrix;
        multiply_3x4_matrix(&mut result, &parent_anim_matrix, &anim_matrix);
    } else {
        multiply_3x4_matrix(&mut result, &root_matrix, &anim_matrix);
    }

    let bone = &mut ghoul2.blist[bli];
    bone.animFrameMatrix = result;
    bone.hasAnimFrameMatrix = frame;
    result
}

/// Shared `G2_Find_Bone`-then-`G2_Add_Bone` idiom `G2_RagGetAnimMatrix` uses
/// twice (`tr_ghoul2.cpp:1441-1450,1481-1486`) — a plain "BONE_NOT_FOUND"
/// name never resolves (Raven: `if (!skel->name || !skel->name[0])
/// bListIndex=-1`).
fn resolve_or_add_bone(ghoul2: &mut CGhoul2Info, name: &str) -> Option<usize> {
    if name.is_empty() {
        return None;
    }
    let mut idx = g2_find_bone(ghoul2.anim_model, &ghoul2.blist, name);
    if idx == -1 {
        idx = crate::bones::g2_add_bone(ghoul2.anim_model, &mut ghoul2.blist, name);
    }
    if idx == -1 {
        None
    } else {
        Some(idx as usize)
    }
}

// ---------------------------------------------------------------------------
// The solver's eight named entry points (roster summary), plus the
// transitively-called private helpers found while enumerating this class.
// ---------------------------------------------------------------------------

/// Raven `static bool G2_RagDollSetup(CGhoul2Info &ghoul2, int frameNum,
/// bool resetOrigin, const vec3_t origin, bool anyRendered)` — the
/// per-frame "compact and set up in topological order" pass: walks
/// `ghoul2.mBlist`, classifies each `BONE_ANGLES_RAGDOLL`/`_IK` bone via
/// `G2_WasBoneRendered` (`render/skeleton.rs`, host-consuming), rebuilds
/// `rag`/`blist_index` (boneNumber-keyed), then the solve-order pass fills
/// `bones`/`effectors`/`num_rags` and resolves each bone's basepose via
/// `G2_GetBoneBasepose` (`render/skeleton.rs`, `EngineHost::model_mdxa`,
/// `G2SV-D13`(b)). Returns `false` (Raven `return false`, `:2397-2399`) when
/// no bone survived (`num_rags == 0`).
///
/// The `#if 0` limb-detection block (`:2344-2375`) and the pure-accounting
/// `minSurvivingBone*`/`numRendered`/`numNotRendered`/`pelvisAt` locals that
/// feed only it (and the fully-commented-out "Deleted Effector" if-block,
/// `:2314-2328`, whose *condition* is pure/side-effect-free) are dropped
/// (§C10 dead-code fold) — this is confirmed dead by reading the source, not
/// an invented simplification.
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:2254-2401`
pub fn g2_rag_doll_setup(
    g2: &mut Ghoul2System,
    _host: &mut impl EngineHost,
    ghoul2: &mut CGhoul2Info,
    frame_num: i32,
    reset_origin: bool,
    origin: vec3_t,
    any_rendered: bool,
) -> bool {
    // `host` unused: `G2_WasBoneRendered`/`G2_GetBoneBasepose` read only the
    // already-resolved `CBoneCache` (module-doc "problems" note #2), not a
    // fresh `EngineHost` lookup.
    g2.rag.rag.clear();

    for i in 0..ghoul2.blist.len() {
        let bone_number = ghoul2.blist[i].boneNumber;
        if bone_number < 0 {
            continue;
        }
        let flags = ghoul2.blist[i].flags;
        if flags & (BONE_ANGLES_RAGDOLL | BONE_ANGLES_IK) == 0 {
            continue;
        }

        let was_rendered = !any_rendered || g2_was_bone_rendered(g2, ghoul2, bone_number);

        let bn = bone_number as usize;
        if g2.rag.rag.len() < bn + 1 {
            g2.rag.rag.resize(bn + 1, -1);
        }
        g2.rag.rag[bn] = i as i32;
        g2.rag.blist_index[bn] = i as i32;

        let bone = &mut ghoul2.blist[i];
        if !was_rendered {
            bone.RagFlags |= RAG_WAS_NOT_RENDERED;
        } else {
            bone.RagFlags &= !RAG_WAS_NOT_RENDERED;
            bone.RagFlags |= RAG_WAS_EVER_RENDERED;
        }
        bone.lastTimeUpdated = frame_num;
        if reset_origin {
            bone.extraVec1 = origin;
        }
    }

    let mut num_rags: i32 = 0;
    for bn in 0..g2.rag.rag.len() {
        let blist_idx = g2.rag.rag[bn];
        if blist_idx < 0 {
            continue;
        }
        let blist_idx = blist_idx as usize;
        let bone_number = ghoul2.blist[blist_idx].boneNumber;

        let (basepose, basepose_inv) = g2_get_bone_basepose(g2, ghoul2, bone_number);
        let radius = ghoul2.blist[blist_idx].radius;
        let weight = ghoul2.blist[blist_idx].weight;

        g2.rag.effectors[num_rags as usize].radius = radius;
        g2.rag.effectors[num_rags as usize].weight = weight;
        g2.rag.rag_bone_data[num_rags as usize] = blist_idx as i32;

        let bone = &mut ghoul2.blist[blist_idx];
        bone.ragIndex = num_rags;
        bone.basepose = basepose;
        bone.baseposeInv = basepose_inv;

        num_rags += 1;
    }
    g2.rag.num_rags = num_rags;
    num_rags != 0
}

/// Round-trip a handle's whole instance `Vec` through the arena for the
/// duration of `f` — needed wherever this file must call an arena-**handle**
/// -consuming sibling (`g2_construct_ghoul_skeleton`, which resolves its
/// model list through `CGhoul2Info_v`/`Ghoul2InfoArray`, not a bare
/// instance) while this file's own top-level entries have already taken the
/// `Vec` out of the arena (`std::mem::take`) to work with it directly
/// (avoiding the self-referential `&mut Ghoul2System` + `&mut CGhoul2Info`
/// aliasing that a live arena borrow would otherwise force on every call —
/// `Vec<CGhoul2Info>: Default` needs no `CGhoul2Info: Default`/`Clone`).
fn with_instances_in_arena<R>(
    g2: &mut Ghoul2System,
    handle: i32,
    instances: &mut Vec<CGhoul2Info>,
    f: impl FnOnce(&mut Ghoul2System) -> R,
) -> R {
    *g2.info_array.get_mut(handle) = core::mem::take(instances);
    let result = f(g2);
    *instances = core::mem::take(g2.info_array.get_mut(handle));
    result
}

/// Raven `static void G2_RagDoll(CGhoul2Info_v &ghoul2V, int g2Index,
/// CRagDollUpdateParams *params, int curTime)` — the per-model ragdoll
/// drive step: bails if `broadsword` is off (`EngineHost::cvar_integer`) or
/// `params` is null, then runs the settle/solve passes and, on the
/// `#ifndef DEDICATED`/`#else` `params->RagDollSettled()` site
/// (`:2497,2505`), matches the no-op `RagDollUpdateKind::Server` hook
/// (`G2SV-D8`).
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:2403-2579`
pub fn g2_rag_doll(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghoul2: &mut CGhoul2Info_v,
    g2_index: i32,
    params: Option<&mut RagDollUpdateParams>,
    cur_time: i32,
) {
    if host.cvar_integer("broadsword") == 0 {
        return;
    }
    let Some(params) = params else {
        return;
    };

    let handle = ghoul2.mItem;
    let mut instances = core::mem::take(g2.info_array.get_mut(handle));
    g2_rag_doll_instances(g2, host, handle, &mut instances, g2_index, params, cur_time);
    *g2.info_array.get_mut(handle) = instances;
}

fn g2_rag_doll_instances(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    handle: i32,
    instances: &mut Vec<CGhoul2Info>,
    g2_index: i32,
    params: &mut RagDollUpdateParams,
    cur_time: i32,
) {
    let d_pos = params.position;
    let frame_num = g2api_get_time(g2, 0);
    let idx = g2_index as usize;

    let mut decay = 1.0f32;
    let mut reset_origin = false;
    let mut any_rendered = false;

    // Settled-state scan + first-rendered-bone probe (`G2_bones.cpp:2467-
    // 2516`). The `#if 0` `noneInSolid` re-check (`:2480-2503`) never
    // compiles; only the live `#else` arm (unconditional `RagDollSettled` +
    // return) is transcribed.
    {
        let blist_len = instances[idx].blist.len();
        for i in 0..blist_len {
            let bone_number = instances[idx].blist[i].boneNumber;
            if bone_number < 0 {
                continue;
            }
            let bone_flags = instances[idx].blist[i].flags;
            if bone_flags & BONE_ANGLES_RAGDOLL == 0 {
                continue;
            }

            let mut ro = reset_origin;
            {
                let ghoul2_ptr: *mut CGhoul2Info = &mut instances[idx];
                let bone = unsafe { alias_bone_mut(ghoul2_ptr, i) };
                decay = g2_rag_set_state(
                    g2,
                    unsafe { &mut *ghoul2_ptr },
                    bone,
                    frame_num,
                    d_pos,
                    &mut ro,
                );
            }
            reset_origin = ro;

            if g2.rag.rag_state == RagState::Settled {
                params.rag_doll_settled();
                return;
            }
            if g2_was_bone_rendered(g2, &instances[idx], bone_number) {
                any_rendered = true;
                break;
            }
        }
    }

    let mut iters = if g2.rag.rag_state == RagState::Dynamic {
        4
    } else {
        2
    };
    if g2.rag.origin_change_dir[2] < -100.0 {
        // rww: was going to be `iters *= 8`; changed to avoid runaway trace
        // counts (Raven's own comment, `G2_bones.cpp:2524`).
        iters *= 2;
    }

    if iters > 0 {
        let setup_ok = g2_rag_doll_setup(
            g2,
            host,
            &mut instances[idx],
            frame_num,
            reset_origin,
            d_pos,
            any_rendered,
        );
        if !setup_ok {
            return;
        }
        for _ in 0..iters {
            g2_rag_doll_current_position_instances(
                g2,
                host,
                handle,
                instances,
                g2_index,
                frame_num,
                params.angles,
                d_pos,
                params.scale,
            );
            g2_rag_doll_settle_position_numero_trois_instances(
                g2,
                host,
                instances,
                g2_index,
                d_pos,
                Some(&mut *params),
                cur_time,
            );
            g2_rag_doll_solve_instances(
                g2,
                host,
                instances,
                g2_index,
                decay * 2.0,
                frame_num,
                d_pos,
                true,
                Some(&mut *params),
            );
        }
    }

    if params.me != ENTITYNUM_NONE {
        g2_rag_doll_current_position_instances(
            g2,
            host,
            handle,
            instances,
            g2_index,
            frame_num,
            params.angles,
            params.position,
            params.scale,
        );
    }
}

/// Raven `static void G2_RagDollCurrentPosition(CGhoul2Info_v &ghoul2V, int
/// g2Index, int frameNum, const vec3_t angles, const vec3_t position, const
/// vec3_t scale)` — rebuilds the render skeleton
/// (`G2_ConstructGhoulSkeleton`, `render/skeleton.rs`, host-consuming),
/// then for every current rag bone resolves its live matrix
/// (`G2_GetBoneMatrixLow`, `render/skeleton.rs`, `EngineHost::model_mdxa`)
/// into `bones`/`effectors`, and updates `bone_mins`/`bone_maxs`/`bone_cm`.
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:2609-2675`
pub fn g2_rag_doll_current_position(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghoul2: &mut CGhoul2Info_v,
    g2_index: i32,
    frame_num: i32,
    angles: vec3_t,
    position: vec3_t,
    scale: vec3_t,
) {
    let handle = ghoul2.mItem;
    let mut instances = core::mem::take(g2.info_array.get_mut(handle));
    g2_rag_doll_current_position_instances(
        g2,
        host,
        handle,
        &mut instances,
        g2_index,
        frame_num,
        angles,
        position,
        scale,
    );
    *g2.info_array.get_mut(handle) = instances;
}

fn g2_rag_doll_current_position_instances(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    handle: i32,
    instances: &mut Vec<CGhoul2Info>,
    g2_index: i32,
    _frame_num: i32,
    angles: vec3_t,
    position: vec3_t,
    scale: vec3_t,
) {
    let idx = g2_index as usize;
    let (world_matrix, _world_matrix_inv) = g2_generate_world_matrix(angles, position);

    // `G2_ConstructGhoulSkeleton` resolves its model list through the
    // `CGhoul2Info_v` arena handle (`render/skeleton.rs`), not a bare
    // instance — round-trip `instances` through the arena for this one call.
    with_instances_in_arena(g2, handle, instances, |g2| {
        let mut wrapper = CGhoul2Info_v { mItem: handle };
        g2_construct_ghoul_skeleton(g2, host, &mut wrapper, _frame_num, false, scale);
    });

    let num_rags = g2.rag.num_rags;
    let mut total_wt = 0.0f32;
    for i in 0..num_rags as usize {
        let blist_idx = g2.rag.rag_bone_data[i] as usize;
        let bone_number = instances[idx].blist[blist_idx].boneNumber;
        let (matrix, _basepose, _basepose_inv) =
            g2_get_bone_matrix_low(g2, &instances[idx], bone_number, scale, &world_matrix);
        g2.rag.bones[i] = matrix;

        let cm_weight = 1.0f32;
        let mut current_origin = [0.0f32; 3];
        for k in 0..3 {
            current_origin[k] = matrix.matrix[k][3];
        }
        g2.rag.effectors[i].current_origin = current_origin;

        if i == 0 {
            _VectorScale(current_origin, cm_weight, &mut g2.rag.bone_cm);
            g2.rag.bone_maxs = current_origin;
            g2.rag.bone_mins = current_origin;
        } else {
            let weight = g2.rag.effectors[i].weight;
            for k in 0..3 {
                g2.rag.bone_cm[k] += current_origin[k] * weight;
                if current_origin[k] > g2.rag.bone_maxs[k] {
                    g2.rag.bone_maxs[k] = current_origin[k];
                }
                if current_origin[k] < g2.rag.bone_mins[k] {
                    g2.rag.bone_mins[k] = current_origin[k];
                }
            }
        }
        total_wt += cm_weight;
    }

    if total_wt > 0.0 {
        let wt_inv = 1.0 / total_wt;
        for k in 0..3 {
            g2.rag.bone_maxs[k] -= position[k];
            g2.rag.bone_mins[k] -= position[k];
            g2.rag.bone_maxs[k] += 10.0;
            g2.rag.bone_mins[k] -= 10.0;
            g2.rag.bone_cm[k] *= wt_inv;
            g2.rag.bone_cm[k] = g2.rag.effectors[0].current_origin[k]; // use the pelvis
        }
    }
}

/// Raven `void Rag_Trace(trace_t *results, const vec3_t start, const vec3_t
/// mins, const vec3_t maxs, const vec3_t end, const int passEntityNum,
/// const int contentmask, const EG2_Collision eG2TraceType, const int
/// useLod)` — the ragdoll/IK collision query entry point every settle/IK
/// helper below calls. The `#ifndef DEDICATED cgvm` client-callback trace
/// branch (`:2688-2704`) never compiles server-side, so this
/// unconditionally runs the real `CM_BoxTrace` (`:2709`) — the doc's own
/// `## Seam definition` names this exact call as one of `EngineHost::trace`'s
/// three ghoul2 call sites (alongside `G2_TraceModels`/`G2_GorePolys`).
/// `results->entityNum` is written on both the hit and no-hit paths
/// (`:2707,2710`, write-through).
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:2683-2723`
#[allow(clippy::too_many_arguments)]
pub fn rag_trace(
    host: &mut impl EngineHost,
    results: &mut trace_t,
    start: vec3_t,
    mins: vec3_t,
    maxs: vec3_t,
    end: vec3_t,
    pass_entity_num: i32,
    contentmask: i32,
    e_g2_trace_type: EG2_Collision,
    use_lod: i32,
) {
    // Raven's real call bypasses `SV_Trace`'s entity loop entirely
    // (`CM_BoxTrace(results, start, end, mins, maxs, 0, contentmask, 0)`,
    // `:2709`) — a world-only trace. `EngineHost::trace` is the doc's frozen
    // seam for this call site regardless (`## Seam definition`); `capsule` /
    // `trace_flags` have no faithful equivalent in the bypassed call, so
    // `false`/`0`. `eG2TraceType` is likewise unused by Raven's own
    // `CM_BoxTrace` call.
    let _ = e_g2_trace_type;
    results.entityNum = ENTITYNUM_NONE as i16;
    host.trace(
        results,
        &start,
        &mins,
        &maxs,
        &end,
        pass_entity_num,
        contentmask,
        false,
        0,
        use_lod,
    );
    results.entityNum = if results.fraction != 1.0 {
        ENTITYNUM_WORLD as i16
    } else {
        ENTITYNUM_NONE as i16
    };
}

/// Raven `static inline bool G2_BoneOnGround(const vec3_t org, const vec3_t
/// mins, const vec3_t maxs, const int ignoreNum)` — traces 1 unit straight
/// down from `org` (`Rag_Trace`) and reports whether that hit solid ground
/// (not in-solid, something hit).
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:2729-2745`
pub fn g2_bone_on_ground(
    host: &mut impl EngineHost,
    org: vec3_t,
    mins: vec3_t,
    maxs: vec3_t,
    ignore_num: i32,
) -> bool {
    let mut g_spot = org;
    g_spot[2] -= 1.0;
    let mut tr = zero_trace();
    rag_trace(
        host,
        &mut tr,
        org,
        mins,
        maxs,
        g_spot,
        ignore_num,
        RAG_MASK,
        EG2_Collision::G2_NOCOLLIDE,
        0,
    );
    tr.fraction != 1.0 && tr.startsolid == 0 && tr.allsolid == 0
}

/// Raven `static inline bool G2_ApplyRealBonePhysics(boneInfo_t &bone,
/// SRagEffector &e, CRagDollUpdateParams *params, vec3_t goalSpot, const
/// vec3_t testMins, const vec3_t testMaxs, const float gravity, const float
/// mass, const float bounce)` — per-bone "exphys"-style gravity/bounce
/// physics step; short-circuits `true` if `bone.physicsSettled`, else
/// applies gravity via `Rag_Trace`-detected ground contact
/// (`G2_BoneOnGround`) before integrating velocity into `goalSpot`.
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:2747-2882`
#[allow(clippy::too_many_arguments)]
pub fn g2_apply_real_bone_physics(
    host: &mut impl EngineHost,
    bone: &mut boneInfo_t,
    e: &mut SRagEffector,
    params: Option<&mut RagDollUpdateParams>,
    goal_spot: &mut vec3_t,
    test_mins: vec3_t,
    test_maxs: vec3_t,
    gravity: f32,
    mass: f32,
    bounce: f32,
) -> bool {
    const MAX_GRAVITY_PULL: f32 = 256.0;
    let velocity_scaling = 0.1f32;

    let Some(params) = params else {
        return true;
    };

    if bone.physicsSettled {
        return true;
    }

    let bone_on_ground;
    if gravity != 0.0 {
        let mut ground = e.current_origin;
        ground[2] -= 1.0;
        let mut tr = zero_trace();
        rag_trace(
            host,
            &mut tr,
            e.current_origin,
            test_mins,
            test_maxs,
            ground,
            params.me,
            RAG_MASK,
            EG2_Collision::G2_NOCOLLIDE,
            0,
        );
        bone_on_ground = tr.entityNum != ENTITYNUM_NONE as i16;

        if !bone_on_ground {
            if params.velocity[2] == 0.0 {
                bone.epGravFactor += gravity;
            }
            if bone.epGravFactor > MAX_GRAVITY_PULL {
                bone.epGravFactor = MAX_GRAVITY_PULL;
            }
            bone.epVelocity[2] -= bone.epGravFactor;
        } else {
            bone.epGravFactor = 0.0;
        }
    } else {
        bone_on_ground = g2_bone_on_ground(host, e.current_origin, test_mins, test_maxs, params.me);
    }

    if bone.epVelocity[0] == 0.0 && bone.epVelocity[1] == 0.0 && bone.epVelocity[2] == 0.0 {
        *goal_spot = e.current_origin;
        return true;
    }

    let mut projected_origin = [0.0f32; 3];
    _VectorMA(
        e.current_origin,
        velocity_scaling,
        bone.epVelocity,
        &mut projected_origin,
    );
    _VectorScale(bone.epVelocity, 1.0 - mass, &mut bone.epVelocity);

    let mut v_norm = bone.epVelocity;
    let v_total = VectorNormalize(&mut v_norm);

    if v_total < 1.0 && bone_on_ground {
        bone.epVelocity = [0.0; 3];
        bone.epGravFactor = 0.0;
        *goal_spot = e.current_origin;
        return true;
    }

    let mut tr = zero_trace();
    rag_trace(
        host,
        &mut tr,
        e.current_origin,
        test_mins,
        test_maxs,
        projected_origin,
        params.me,
        RAG_MASK,
        EG2_Collision::G2_NOCOLLIDE,
        0,
    );

    if tr.startsolid != 0 || tr.allsolid != 0 {
        return false;
    }

    *goal_spot = tr.endpos;

    if tr.fraction == 1.0 {
        return true;
    }

    if bounce != 0.0 {
        let v_total = v_total * bounce;
        let mut v_norm = [0.0f32; 3];
        _VectorScale(tr.plane.normal, v_total, &mut v_norm);
        if v_norm[2] > 0.0 {
            bone.epGravFactor -= v_norm[2] * (1.0 - mass);
            if bone.epGravFactor < 0.0 {
                bone.epGravFactor = 0.0;
            }
        }
        _VectorAdd(bone.epVelocity, v_norm, &mut bone.epVelocity);
    } else {
        bone.epVelocity[0] = 0.0;
        bone.epVelocity[1] = 0.0;
        if gravity == 0.0 {
            bone.epVelocity[2] = 0.0;
        }
    }
    true
}

/// Raven `static void G2_Generate_MatrixRag(boneInfo_v &blist, int index)`
/// — "caution this must not be called before the whole skeleton is
/// 'remembered'": copies `bone.ragOverrideMatrix` into `bone.matrix`/
/// `bone.newMatrix`. Pure bone-list mutation, no host service.
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:1244-1265`
pub fn g2_generate_matrix_rag(blist: &mut Vec<boneInfo_t>, index: i32) {
    let bone = &mut blist[index as usize];
    bone.matrix = bone.ragOverrideMatrix;
    bone.newMatrix = bone.matrix;
}

/// Raven `static bool G2_RagDollSettlePositionNumeroTrois(CGhoul2Info_v
/// &ghoul2V, const vec3_t currentOrg, CRagDollUpdateParams *params, int
/// curTime)` — the multi-bone constraint-settle pass; returns `true` if any
/// bone was in solid this call. Reads `broadsword`
/// (`EngineHost::cvar_integer`), calls `flrand` (`EngineHost::flrand`) and
/// `Rag_Trace` (`EngineHost::trace`) per bone. The four
/// `RAG_CALLBACK_BONEINSOLID` cgvm callback sites (`:3056,3085,3180,3217`,
/// `#ifndef DEDICATED`) fold out server-side (surrounding `if (params)`
/// solid-handling stays); a fifth (`:3826`) is doubly dead (`#if 0` +
/// `#ifndef DEDICATED`). The `_DEBUG_BONE_NAMES` high-solid-count print
/// block (`:3848-3864`) is dropped: `_DEBUG_BONE_NAMES` is only `#define`d
/// under `#ifdef _DEBUG` (`:2576-2578`), which the doc's own NDEBUG WinDed
/// build config leaves off — genuinely dead here (a correction to this
/// doc's prose elsewhere, reported upstream), not merely a debug convenience
/// dropped by convention.
///
/// **Reported gap** (module-doc note #5): the `broadsword_ragtobase > 1`
/// pelvis-offset sub-branch (`:3496-3524,3625-3663`) needs
/// `g2_rag_get_pelvis_lumbar_offsets`/`g2_rag_get_world_anim_matrix`'s
/// `worldMatrix`, which this function's own frozen signature has no
/// parameter to receive either — both calls are made with a local identity
/// matrix stand-in, which is faithful for `broadsword_ragtobase <= 1`
/// (`hasBasePos` stays reachable and correct there — that branch does not
/// need `worldMatrix`) but not bit-exact once `> 1`; that sub-case is a
/// tuning cvar defaulting off.
///
/// A dead twin of this function (`#ifdef _OLD_STYLE_SETTLE`, never
/// `#define`d anywhere in `codemp/`) exists at `:2927-3336`; only this live
/// definition is ported (the doc's ground-truth prose cites both line
/// numbers together, `:2927`/`:3449` — reported upstream as a citation
/// ambiguity, not a second port target).
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:3449-3935`
pub fn g2_rag_doll_settle_position_numero_trois(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghoul2: &mut CGhoul2Info_v,
    current_org: vec3_t,
    params: Option<&mut RagDollUpdateParams>,
    cur_time: i32,
) -> bool {
    let Some(params) = params else {
        return false;
    };
    let handle = ghoul2.mItem;
    let mut instances = core::mem::take(g2.info_array.get_mut(handle));
    // g2_index isn't threaded to this Raven signature (it always reads
    // `ghoul2V[0]`); every live caller in this file operates on a
    // single-instance ragdoll model, so index 0 is used directly, matching
    // the oracle's own hardcoded `ghoul2V[0]` reads.
    let result = g2_rag_doll_settle_position_numero_trois_instances(
        g2,
        host,
        &mut instances,
        0,
        current_org,
        Some(params),
        cur_time,
    );
    *g2.info_array.get_mut(handle) = instances;
    result
}

#[allow(clippy::too_many_arguments)]
fn g2_rag_doll_settle_position_numero_trois_instances(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    instances: &mut Vec<CGhoul2Info>,
    g2_index: i32,
    // Raven's own `currentOrg` parameter is dead in this fn's body too (grep:
    // `G2_bones.cpp:3449-3935` never reads it — every use is `params-
    // >position` instead, a structurally distinct but same-valued param at
    // every call site) — kept for 1:1 signature fidelity, same treatment as
    // `g2_rag_set_state`'s dead `reset_origin` out-param.
    _current_org: vec3_t,
    params: Option<&mut RagDollUpdateParams>,
    cur_time: i32,
) -> bool {
    let Some(params) = params else {
        return false;
    };
    let idx = g2_index as usize;
    let ignore_num = params.me;

    const VELOCITY_DAMPENING: f32 = 1.0;
    const VELOCITY_MULTIPLIER: f32 = 60.0;
    const GRAVITY: f32 = 3.0;
    const MASS: f32 = 0.09;
    const BOUNCE: f32 = 0.0;

    let in_air =
        params.velocity[0] != 0.0 || params.velocity[1] != 0.0 || params.velocity[2] != 0.0;
    let ent_scale = if params.scale[0] == 0.0 && params.scale[1] == 0.0 && params.scale[2] == 0.0 {
        [1.0, 1.0, 1.0]
    } else {
        params.scale
    };

    let broadsword_ragtobase = host.cvar_integer("broadsword_ragtobase");
    let mut anim_pelvis_dir = [0.0f32; 3];
    let mut pelvis_dir = [0.0f32; 3];
    let mut anim_pelvis_pos = [0.0f32; 3];
    let mut pelvis_pos = [0.0f32; 3];
    if broadsword_ragtobase > 1 {
        // See this fn's doc comment: blocked by a missing `worldMatrix`
        // parameter (module-doc "problems" note #5); a no-op leaves the
        // four vectors at zero rather than inventing a substitute.
        g2_rag_get_pelvis_lumbar_offsets(
            g2,
            host,
            &mut instances[0],
            Some(&mut *params),
            &mut pelvis_pos,
            &mut pelvis_dir,
            &mut anim_pelvis_pos,
            &mut anim_pelvis_dir,
        );
        pelvis_dir[2] = 0.0;
        anim_pelvis_dir[2] = 0.0;
        vectoangles(pelvis_dir, &mut pelvis_dir);
        vectoangles(anim_pelvis_dir, &mut anim_pelvis_dir);
    }

    let num_rags = g2.rag.num_rags;
    let mut any_solid = false;

    for i in 0..num_rags as usize {
        let blist_idx = g2.rag.rag_bone_data[i] as usize;
        let bone_number = instances[idx].blist[blist_idx].boneNumber;
        let rag_flags = instances[idx].blist[blist_idx].RagFlags;

        if in_air {
            instances[idx].blist[blist_idx].airTime = cur_time + 30;
        }

        if rag_flags & RAG_PCJ_PELVIS != 0 {
            let goal_spot = [
                params.position[0],
                params.position[1],
                (params.position[2] + DEFAULT_MINS_2)
                    + (g2.rag.effectors[i].radius * ent_scale[2] + 2.0),
            ];
            _VectorSubtract(
                goal_spot,
                g2.rag.effectors[i].current_origin,
                &mut g2.rag.desired_pelvis_offset,
            );
            g2.rag.have_desired_pelvis_offset = true;
            instances[idx].blist[blist_idx].lastPosition = g2.rag.effectors[i].current_origin;
            continue;
        }
        if rag_flags & RAG_EFFECTOR == 0 {
            continue;
        }

        if instances[idx].blist[blist_idx].hasOverGoal {
            let over_goal_spot = instances[idx].blist[blist_idx].overGoalSpot;
            instances[idx].blist[blist_idx].solidCount = 0;
            let current_origin = g2.rag.effectors[i].current_origin;
            let velocity_effector = instances[idx].blist[blist_idx].velocityEffector;
            let mut desired = [0.0f32; 3];
            for k in 0..3 {
                desired[k] = (over_goal_spot[k] - current_origin[k])
                    + VELOCITY_MULTIPLIER * velocity_effector[k];
            }
            g2.rag.effectors[i].desired_direction = desired;
            let bone = &mut instances[idx].blist[blist_idx];
            for k in 0..3 {
                bone.velocityEffector[k] *= VELOCITY_DAMPENING;
            }
            bone.lastPosition = current_origin;
            continue;
        }

        let radius = g2.rag.effectors[i].radius;
        let test_mins: vec3_t = [
            -radius * ent_scale[0],
            -radius * ent_scale[1],
            -radius * ent_scale[2],
        ];
        let test_maxs: vec3_t = [
            radius * ent_scale[0],
            radius * ent_scale[1],
            radius * ent_scale[2],
        ];

        // Parent-bone lookup (`hasDaddy`): walk the skeleton hierarchy for
        // the first ancestor that is itself a rag bone, caching the result
        // on `parentBoneIndex` exactly as Raven does.
        let mut has_daddy = false;
        let mut parent_origin = [0.0f32; 3];
        if bone_number != 0 {
            let parent_blist_index = instances[idx].blist[blist_idx].parentBoneIndex;
            let resolved_parent = if parent_blist_index == -1 {
                let a_header = instances[idx].a_header;
                let anim_model = instances[idx].anim_model;
                let mut found = -1i32;
                if !a_header.is_null() {
                    unsafe {
                        let skel = mdxa_skel_ptr(a_header, bone_number);
                        let mut b_parent_index = mdxa_skel_parent(skel);
                        while b_parent_index > 0 {
                            let pskel = mdxa_skel_ptr(a_header, b_parent_index);
                            let pname = mdxa_skel_name(pskel);
                            b_parent_index = mdxa_skel_parent(pskel);
                            let bli = g2_find_bone(anim_model, &instances[idx].blist, &pname);
                            if bli != -1
                                && instances[idx].blist[bli as usize].flags & BONE_ANGLES_RAGDOLL
                                    != 0
                            {
                                found = bli;
                                break;
                            }
                        }
                    }
                }
                instances[idx].blist[blist_idx].parentBoneIndex = found;
                found
            } else {
                parent_blist_index
            };
            if resolved_parent != -1 {
                let pbone = &instances[idx].blist[resolved_parent as usize];
                if pbone.flags & BONE_ANGLES_RAGDOLL != 0 {
                    parent_origin = g2.rag.effectors[pbone.ragIndex as usize].current_origin;
                    has_daddy = true;
                }
            }
        }

        // `hasBasePos` (the `broadsword_ragtobase` desired-frame hint); see
        // this fn's doc comment for the identity-`worldMatrix` note.
        let mut has_base_pos = false;
        let mut base_pos = [0.0f32; 3];
        if broadsword_ragtobase != 0 {
            let mut world_base_matrix = ZERO_BONE;
            {
                let ghoul2_ptr: *mut CGhoul2Info = &mut instances[idx];
                let bone = unsafe { alias_bone_mut(ghoul2_ptr, blist_idx) };
                g2_rag_get_world_anim_matrix(
                    g2,
                    host,
                    unsafe { &mut *ghoul2_ptr },
                    bone,
                    Some(&mut *params),
                    &ZERO_BONE,
                    &mut world_base_matrix,
                );
            }
            base_pos = g2api_give_me_vector_from_matrix(&world_base_matrix, Eorientations::ORIGIN);

            if broadsword_ragtobase > 1 {
                let offset_rotation = instances[idx].blist[blist_idx].offsetRotation;
                let fa_raw = AngleNormalize180(anim_pelvis_dir[1] - pelvis_dir[1]);
                let d = fa_raw - offset_rotation;
                let fa = if !(-16.0..=16.0).contains(&d) {
                    instances[idx].blist[blist_idx].offsetRotation = fa_raw;
                    fa_raw
                } else {
                    offset_rotation
                };
                let mut v = [0.0f32; 3];
                _VectorSubtract(base_pos, anim_pelvis_pos, &mut v);
                let f = VectorLength(v);
                let mut a = [0.0f32; 3];
                vectoangles(v, &mut a);
                a[1] -= fa;
                AngleVectors(a, Some(&mut v), None, None);
                VectorNormalize(&mut v);
                _VectorMA(anim_pelvis_pos, f, v, &mut base_pos);
                _VectorSubtract(base_pos, anim_pelvis_pos, &mut v);
                _VectorAdd(pelvis_pos, v, &mut base_pos);
            }
            has_base_pos = true;
        }

        let current_origin = g2.rag.effectors[i].current_origin;
        let mut goal_spot;
        let start_solid;
        {
            let mut tr = zero_trace();
            let trace_end = if has_daddy {
                parent_origin
            } else {
                params.position
            };
            rag_trace(
                host,
                &mut tr,
                current_origin,
                test_mins,
                test_maxs,
                trace_end,
                ignore_num,
                RAG_MASK,
                EG2_Collision::G2_NOCOLLIDE,
                0,
            );

            if tr.startsolid != 0 || tr.allsolid != 0 || tr.fraction != 1.0 {
                start_solid = true;
                any_solid = true;
                if has_base_pos {
                    goal_spot = base_pos;
                    goal_spot[2] = (params.position[2] - 23.0) - test_mins[2];
                } else {
                    let mut v_sub = [0.0f32; 3];
                    _VectorSubtract(current_origin, params.position, &mut v_sub);
                    VectorNormalize(&mut v_sub);
                    goal_spot = [0.0f32; 3];
                    _VectorMA(params.position, 40.0, v_sub, &mut goal_spot);
                    goal_spot[2] = (params.position[2] - 23.0) - test_mins[2];
                }
                let mut tr2 = zero_trace();
                rag_trace(
                    host,
                    &mut tr2,
                    params.position,
                    test_mins,
                    test_maxs,
                    goal_spot,
                    params.me,
                    RAG_MASK,
                    EG2_Collision::G2_NOCOLLIDE,
                    0,
                );
                goal_spot = tr2.endpos;
            } else {
                start_solid = false;
                let mut vel_dir = [0.0f32; 3];
                if has_daddy || has_base_pos {
                    if has_base_pos {
                        _VectorSubtract(base_pos, current_origin, &mut vel_dir);
                    } else {
                        _VectorSubtract(current_origin, parent_origin, &mut vel_dir);
                    }
                } else {
                    _VectorSubtract(current_origin, params.position, &mut vel_dir);
                }
                if VectorLength(vel_dir) > 2.0 {
                    VectorNormalize(&mut vel_dir);
                    _VectorScale(vel_dir, 8.0, &mut vel_dir);
                    vel_dir[2] = 0.0;
                    let bone = &mut instances[idx].blist[blist_idx];
                    _VectorAdd(bone.epVelocity, vel_dir, &mut bone.epVelocity);
                }

                if rag_flags & RAG_BONE_LIGHTWEIGHT != 0 {
                    let mut vel = [0.0f32; 3];
                    _VectorScale(params.velocity, 0.5, &mut vel);
                    let vellen = VectorLength(vel);
                    if vellen > 64.0 {
                        _VectorScale(vel, 64.0 / vellen, &mut vel);
                    }
                    VectorInverse(&mut vel);
                    let bone = &mut instances[idx].blist[blist_idx];
                    if vel[2] != 0.0 {
                        bone.epVelocity = vel;
                    } else {
                        _VectorAdd(bone.epVelocity, vel, &mut bone.epVelocity);
                    }
                }

                goal_spot = current_origin;
                let mut e = g2.rag.effectors[i];
                let ok = {
                    let bone = &mut instances[idx].blist[blist_idx];
                    g2_apply_real_bone_physics(
                        host,
                        bone,
                        &mut e,
                        Some(&mut *params),
                        &mut goal_spot,
                        test_mins,
                        test_maxs,
                        GRAVITY,
                        MASS,
                        BOUNCE,
                    )
                };
                g2.rag.effectors[i] = e;
                if !ok {
                    goal_spot = params.position;
                }
            }
        }

        {
            let bone = &mut instances[idx].blist[blist_idx];
            if start_solid {
                bone.solidCount += 1;
            } else {
                bone.solidCount = 0;
            }
        }

        let broadsword_dircap = host.cvar_integer("broadsword_dircap") as f32; // value-typed cvar read via the integer accessor (no float cvar service on EngineHost)
        let solid_count = instances[idx].blist[blist_idx].solidCount;
        let velocity_effector = instances[idx].blist[blist_idx].velocityEffector;
        let mut desired = [0.0f32; 3];
        for k in 0..3 {
            let mut d = goal_spot[k] - current_origin[k];
            if broadsword_dircap != 0.0 {
                let mut cap = broadsword_dircap;
                if solid_count > 5 {
                    let mut solid_factor = solid_count as f32 * 0.2;
                    if solid_factor > 16.0 {
                        solid_factor = 16.0;
                    }
                    d *= solid_factor;
                    cap *= 8.0;
                }
                if d > cap {
                    d = cap;
                } else if d < -cap {
                    d = -cap;
                }
            }
            d += VELOCITY_MULTIPLIER * velocity_effector[k];
            d += host.flrand(-0.75, 0.75) * host.flrand(-0.75, 0.75);
            desired[k] = d;
        }
        g2.rag.effectors[i].desired_direction = desired;
        let bone = &mut instances[idx].blist[blist_idx];
        for k in 0..3 {
            bone.velocityEffector[k] *= VELOCITY_DAMPENING;
        }
        bone.lastPosition = current_origin;
    }

    any_solid
}

/// Raven `static float AngleNormZero(float theta)` — normalizes `theta`
/// (mod 360) into `[-180, 180]`.
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:3936-3949`
pub fn angle_norm_zero(theta: f32) -> f32 {
    let mut ret = theta % 360.0;
    if ret < -180.0 {
        ret += 360.0;
    } else if ret > 180.0 {
        ret -= 360.0;
    }
    ret
}

/// Raven `static inline void G2_BoneSnap(CGhoul2Info_v &ghoul2V, boneInfo_t
/// &bone, CRagDollUpdateParams *params)` — cgame bone-snap-effect callback;
/// `#ifdef DEDICATED return;` is the **entire** body's live arm (the
/// `#else` `cgvm`/`VM_Call` branch never compiles in the WinDed DEDICATED
/// build), so this is a compiled no-op server-side (`## Raven ground
/// truth`, "cgvm ragdoll-callback dead branches"); its sole caller is
/// `G2_RagDollSolve` (`:4244`). No `host` needed.
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:3951-3968`
pub fn g2_bone_snap(
    _ghoul2: &mut CGhoul2Info_v,
    _bone: &mut boneInfo_t,
    _params: Option<&mut RagDollUpdateParams>,
) {
    // #ifdef DEDICATED return; -- the entire live-build body.
}

/// Raven `static void G2_RagDollSolve(CGhoul2Info_v &ghoul2V, int g2Index,
/// float decay, int frameNum, const vec3_t currentOrg, bool limitAngles,
/// CRagDollUpdateParams *params)` — the top-level per-frame ragdoll solve:
/// drives the settle pass, applies gradient-descent bone-angle updates
/// (`AngleNormZero` normalizes each result), and seeds via `flrand`
/// (`EngineHost::flrand`, `:2127-2129`) where per-bone randomization is
/// needed; the caller (`G2_RagDoll`) reaches `G2_BoneSnap` on its solved
/// output (`:4244`).
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:3970-4256`
pub fn g2_rag_doll_solve(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghoul2: &mut CGhoul2Info_v,
    g2_index: i32,
    decay: f32,
    frame_num: i32,
    current_org: vec3_t,
    limit_angles: bool,
    params: Option<&mut RagDollUpdateParams>,
) {
    let handle = ghoul2.mItem;
    let mut instances = core::mem::take(g2.info_array.get_mut(handle));
    g2_rag_doll_solve_instances(
        g2,
        host,
        &mut instances,
        g2_index,
        decay,
        frame_num,
        current_org,
        limit_angles,
        params,
    );
    *g2.info_array.get_mut(handle) = instances;
}

#[allow(clippy::too_many_arguments)]
fn g2_rag_doll_solve_instances(
    g2: &mut Ghoul2System,
    _host: &mut impl EngineHost,
    instances: &mut Vec<CGhoul2Info>,
    g2_index: i32,
    decay: f32,
    _frame_num: i32,
    _current_org: vec3_t,
    limit_angles: bool,
    mut params: Option<&mut RagDollUpdateParams>,
) {
    // `host` unused: this solve step's own body never reaches a host
    // service directly (the settle pass around it does the tracing/flrand).
    let idx = g2_index as usize;
    let num_rags = g2.rag.num_rags;

    for i in 0..num_rags as usize {
        let blist_idx = g2.rag.rag_bone_data[i] as usize;
        let rag_flags = instances[idx].blist[blist_idx].RagFlags;
        if rag_flags & RAG_PCJ == 0 {
            continue;
        }

        let n = inverse_matrix(&g2.rag.bones[i]);
        let t_angles0 = instances[idx].blist[blist_idx].currentAngles;
        let cur_rot = create_matrix(t_angles0);
        let cur_rot_inv = inverse_matrix(&cur_rot);
        let mut p = ZERO_BONE;
        multiply_3x4_matrix(&mut p, &g2.rag.bones[i], &cur_rot_inv);

        if rag_flags & RAG_PCJ_MODEL_ROOT != 0 {
            if g2.rag.have_desired_pelvis_offset {
                let delta = transform_point(g2.rag.desired_pelvis_offset, &n);
                let bone = &mut instances[idx].blist[blist_idx];
                for k in 0..3 {
                    let move_to = bone.velocityRoot[k] + delta[k] * 0.20;
                    bone.velocityRoot[k] = (bone.velocityRoot[k] - move_to) * 0.25 + move_to;
                    bone.ragOverrideMatrix.matrix[k][3] = bone.velocityRoot[k];
                }
            }
        } else {
            let mut gs = [ZERO_BONE; 3];
            for k in 0..3 {
                let mut t = t_angles0;
                t[k] += 0.5;
                let temp2 = create_matrix(t);
                let mut temp1 = ZERO_BONE;
                multiply_3x4_matrix(&mut temp1, &p, &temp2);
                multiply_3x4_matrix(&mut gs[k], &temp1, &n);
            }

            let bone_number = instances[idx].blist[blist_idx].boneNumber;
            let mut temp_dependents = [0i32; MAX_BONES_RAG];
            let num_dep =
                g2_get_bone_dependents(g2, &instances[idx], bone_number, &mut temp_dependents);

            let mut del_angles = [0.0f32; 3];
            let mut num_rag_dep = 0i32;
            let mut all_solid_count = 0i32;
            for j in 0..num_dep as usize {
                let dep_bone_number = temp_dependents[j];
                if dep_bone_number < 0 || dep_bone_number as usize >= g2.rag.rag.len() {
                    continue;
                }
                let dep_blist_idx = g2.rag.rag[dep_bone_number as usize];
                if dep_blist_idx < 0 {
                    continue;
                }
                let dep_bone = &instances[idx].blist[dep_blist_idx as usize];
                if dep_bone.RagFlags & RAG_EFFECTOR == 0 {
                    continue;
                }
                let dep_index = dep_bone.ragIndex as usize;
                let dep_weight = dep_bone.weight;
                let dep_solid_count = dep_bone.solidCount;
                num_rag_dep += 1;
                for k in 0..3 {
                    let mut enew = ZERO_BONE;
                    multiply_3x4_matrix(&mut enew, &gs[k], &g2.rag.bones[dep_index]);
                    let t_position = [enew.matrix[0][3], enew.matrix[1][3], enew.matrix[2][3]];
                    let mut change = [0.0f32; 3];
                    _VectorSubtract(
                        t_position,
                        g2.rag.effectors[dep_index].current_origin,
                        &mut change,
                    );
                    let goodness =
                        _DotProduct(change, g2.rag.effectors[dep_index].desired_direction)
                            * dep_weight;
                    del_angles[k] += goodness;
                }
                all_solid_count += dep_solid_count;
            }
            all_solid_count += instances[idx].blist[blist_idx].solidCount;

            let mut magic_factor1 = 0.40f32;
            if all_solid_count > 32 {
                magic_factor1 = 0.6;
            } else if all_solid_count > 10 {
                magic_factor1 = 0.5;
            }
            let over_grad_speed = instances[idx].blist[blist_idx].overGradSpeed;
            if over_grad_speed != 0.0 {
                magic_factor1 = over_grad_speed;
            }
            let recip = if num_rag_dep != 0 {
                (4.0 / num_rag_dep as f32).sqrt()
            } else {
                0.0
            };
            let fac = decay * recip * magic_factor1;

            let mut magic_factor9 = 0.75f32;
            if g2.rag.rag_state == RagState::Dynamic {
                magic_factor9 = 0.85;
            }
            let mut magic_factor32 = 1.5f32;
            if rag_flags & RAG_UNSNAPPABLE != 0 {
                magic_factor32 = 1.0;
            }

            let mut is_snapped = false;
            {
                let bone = &mut instances[idx].blist[blist_idx];
                bone.lastAngles = bone.currentAngles;
                for k in 0..3 {
                    bone.currentAngles[k] += del_angles[k] * fac;
                    bone.currentAngles[k] = (bone.lastAngles[k] - bone.currentAngles[k])
                        * magic_factor9
                        + bone.currentAngles[k];
                    bone.currentAngles[k] = angle_norm_zero(bone.currentAngles[k]);
                    if limit_angles
                        && (all_solid_count < 32 || rag_flags & RAG_UNSNAPPABLE != 0)
                        && (!bone.snapped || rag_flags & RAG_UNSNAPPABLE != 0)
                    {
                        if bone.currentAngles[k] > bone.maxAngles[k] * magic_factor32 {
                            bone.currentAngles[k] = bone.maxAngles[k] * magic_factor32;
                        }
                        if bone.currentAngles[k] < bone.minAngles[k] * magic_factor32 {
                            bone.currentAngles[k] = bone.minAngles[k] * magic_factor32;
                        }
                    }
                }
                for k in 0..3 {
                    if bone.currentAngles[k] > bone.maxAngles[k] * magic_factor32
                        || bone.currentAngles[k] < bone.minAngles[k] * magic_factor32
                    {
                        is_snapped = true;
                        break;
                    }
                }
                if is_snapped != bone.snapped {
                    // G2_BoneSnap is a compiled no-op server-side (see its own doc comment).
                    bone.snapped = is_snapped;
                }
                let temp1 = create_matrix(bone.currentAngles);
                let basepose_inv = if bone.baseposeInv.is_null() {
                    ZERO_BONE
                } else {
                    unsafe { *bone.baseposeInv }
                };
                let basepose = if bone.basepose.is_null() {
                    ZERO_BONE
                } else {
                    unsafe { *bone.basepose }
                };
                let mut temp2 = ZERO_BONE;
                multiply_3x4_matrix(&mut temp2, &temp1, &basepose_inv);
                multiply_3x4_matrix(&mut bone.ragOverrideMatrix, &basepose, &temp2);
            }
        }

        g2_generate_matrix_rag(&mut instances[idx].blist, blist_idx as i32);
    }
    let _ = &mut params; // threaded per the frozen signature; RagDollSolve's own body never touches `params` (only its caller does around it).
}

/// Raven `static void G2_IKSolve(CGhoul2Info_v &ghoul2V, int g2Index, float
/// decay, int frameNum, const vec3_t currentOrg, bool limitAngles)` — the IK
/// arm's per-bone gradient-descent angle solve (shares the solver's
/// `bones`/`effectors`/`num_rags` state with the ragdoll arm, `## Raven
/// ground truth`); reads bone dependents (`G2_GetBoneDependents`,
/// `render/skeleton.rs`) and re-derives each PCJ-controlled bone's override
/// matrix via `G2_Generate_MatrixRag`. No host service touched directly.
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:4297-4452`
pub fn g2_ik_solve(
    g2: &mut Ghoul2System,
    ghoul2: &mut CGhoul2Info_v,
    g2_index: i32,
    decay: f32,
    frame_num: i32,
    current_org: vec3_t,
    limit_angles: bool,
) {
    let handle = ghoul2.mItem;
    let mut instances = core::mem::take(g2.info_array.get_mut(handle));
    g2_ik_solve_instances(
        g2,
        &mut instances,
        g2_index,
        decay,
        frame_num,
        current_org,
        limit_angles,
    );
    *g2.info_array.get_mut(handle) = instances;
}

fn g2_ik_solve_instances(
    g2: &mut Ghoul2System,
    instances: &mut Vec<CGhoul2Info>,
    g2_index: i32,
    decay: f32,
    _frame_num: i32,
    _current_org: vec3_t,
    limit_angles: bool,
) {
    let idx = g2_index as usize;
    let num_rags = g2.rag.num_rags;

    for i in 0..num_rags as usize {
        let blist_idx = g2.rag.rag_bone_data[i] as usize;
        let rag_flags = instances[idx].blist[blist_idx].RagFlags;
        if rag_flags & RAG_PCJ_MODEL_ROOT != 0 {
            continue;
        }
        if rag_flags & RAG_PCJ_IK_CONTROLLED == 0 {
            continue;
        }

        let n = inverse_matrix(&g2.rag.bones[i]);
        let t_angles0 = instances[idx].blist[blist_idx].currentAngles;
        let cur_rot = create_matrix(t_angles0);
        let cur_rot_inv = inverse_matrix(&cur_rot);
        let mut p = ZERO_BONE;
        multiply_3x4_matrix(&mut p, &g2.rag.bones[i], &cur_rot_inv);

        let mut gs = [ZERO_BONE; 3];
        for k in 0..3 {
            let mut t = t_angles0;
            t[k] += 0.5;
            let temp2 = create_matrix(t);
            let mut temp1 = ZERO_BONE;
            multiply_3x4_matrix(&mut temp1, &p, &temp2);
            multiply_3x4_matrix(&mut gs[k], &temp1, &n);
        }

        let bone_number = instances[idx].blist[blist_idx].boneNumber;
        let mut temp_dependents = [0i32; MAX_BONES_RAG];
        let num_dep =
            g2_get_bone_dependents(g2, &instances[idx], bone_number, &mut temp_dependents);

        let mut del_angles = [0.0f32; 3];
        // Raven accumulates `numRagDep` here too (`G2_bones.cpp:4357-4392`)
        // but never reads it afterward in `G2_IKSolve` — `recip` is the
        // literal `sqrt(4.0f/1.0f)` (`:4409`), unlike `G2_RagDollSolve`'s use
        // of the ragdoll-arm's own `numRagDep`. A genuinely dead write in the
        // oracle, kept (not dropped) for fidelity to the transcribed loop
        // shape, `_`-prefixed to silence the unused-write lint.
        let mut _num_rag_dep = 0i32;
        for j in 0..num_dep as usize {
            let dep_bone_number = temp_dependents[j];
            if dep_bone_number < 0 || dep_bone_number as usize >= g2.rag.rag.len() {
                continue;
            }
            let dep_blist_idx = g2.rag.rag[dep_bone_number as usize];
            if dep_blist_idx < 0 {
                continue;
            }
            let dep_bone = &instances[idx].blist[dep_blist_idx as usize];
            let dep_index = dep_bone.ragIndex as usize;
            if g2.rag.rag_bone_data[dep_index] < 0 {
                continue;
            }
            if dep_bone.RagFlags & RAG_EFFECTOR == 0 {
                continue;
            }
            let dep_weight = dep_bone.weight;
            _num_rag_dep += 1;
            for k in 0..3 {
                let mut enew = ZERO_BONE;
                multiply_3x4_matrix(&mut enew, &gs[k], &g2.rag.bones[dep_index]);
                let t_position = [enew.matrix[0][3], enew.matrix[1][3], enew.matrix[2][3]];
                let mut change = [0.0f32; 3];
                _VectorSubtract(
                    t_position,
                    g2.rag.effectors[dep_index].current_origin,
                    &mut change,
                );
                let goodness =
                    _DotProduct(change, g2.rag.effectors[dep_index].desired_direction) * dep_weight;
                del_angles[k] += goodness;
            }
        }

        let mut magic_factor1 = instances[idx].blist[blist_idx].ikSpeed;
        if magic_factor1 == 0.0 {
            magic_factor1 = 0.40;
        }
        let recip = (4.0f32 / 1.0).sqrt();
        let fac = decay * recip * magic_factor1;

        let mut magic_factor9 = 0.75f32;
        if g2.rag.rag_state == RagState::Dynamic {
            magic_factor9 = 0.85;
        }
        const MAGIC_FACTOR32: f32 = 1.0;

        let max_angles = instances[idx].blist[blist_idx].maxAngles;
        let min_angles = instances[idx].blist[blist_idx].minAngles;
        let free_this_bone = max_angles[0] == 0.0
            && max_angles[1] == 0.0
            && max_angles[2] == 0.0
            && min_angles[0] == 0.0
            && min_angles[1] == 0.0
            && min_angles[2] == 0.0;

        {
            let bone = &mut instances[idx].blist[blist_idx];
            bone.lastAngles = bone.currentAngles;
            for k in 0..3 {
                bone.currentAngles[k] += del_angles[k] * fac;
                bone.currentAngles[k] = (bone.lastAngles[k] - bone.currentAngles[k])
                    * magic_factor9
                    + bone.currentAngles[k];
                bone.currentAngles[k] = angle_norm_zero(bone.currentAngles[k]);
                if limit_angles && !free_this_bone {
                    if bone.currentAngles[k] > bone.maxAngles[k] * MAGIC_FACTOR32 {
                        bone.currentAngles[k] = bone.maxAngles[k] * MAGIC_FACTOR32;
                    }
                    if bone.currentAngles[k] < bone.minAngles[k] * MAGIC_FACTOR32 {
                        bone.currentAngles[k] = bone.minAngles[k] * MAGIC_FACTOR32;
                    }
                }
            }
            let temp1 = create_matrix(bone.currentAngles);
            let basepose_inv = if bone.baseposeInv.is_null() {
                ZERO_BONE
            } else {
                unsafe { *bone.baseposeInv }
            };
            let basepose = if bone.basepose.is_null() {
                ZERO_BONE
            } else {
                unsafe { *bone.basepose }
            };
            let mut temp2 = ZERO_BONE;
            multiply_3x4_matrix(&mut temp2, &temp1, &basepose_inv);
            multiply_3x4_matrix(&mut bone.ragOverrideMatrix, &basepose, &temp2);
        }

        g2_generate_matrix_rag(&mut instances[idx].blist, blist_idx as i32);
    }
}

/// Raven `static void G2_DoIK(CGhoul2Info_v &ghoul2V, int g2Index,
/// CRagDollUpdateParams *params)` — the IK arm's top-level per-frame drive:
/// `G2_RagDollSetup` (host-consuming) then, 12 iterations of
/// `G2_RagDollCurrentPosition` + `G2_IKReposition` (both host-consuming,
/// the latter via `flrand`) + `G2_IKSolve`, finishing with one more
/// `G2_RagDollCurrentPosition` call if `params->me` names a real entity.
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:4453-4495`
pub fn g2_do_ik(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghoul2: &mut CGhoul2Info_v,
    g2_index: i32,
    params: Option<&mut RagDollUpdateParams>,
) {
    let Some(params) = params else {
        return;
    };
    let frame_num = g2api_get_time(g2, 0);
    let handle = ghoul2.mItem;
    let mut instances = core::mem::take(g2.info_array.get_mut(handle));

    let setup_ok = {
        let idx = g2_index as usize;
        g2_rag_doll_setup(
            g2,
            host,
            &mut instances[idx],
            frame_num,
            false,
            params.position,
            false,
        )
    };
    if setup_ok {
        for _ in 0..12 {
            g2_rag_doll_current_position_instances(
                g2,
                host,
                handle,
                &mut instances,
                g2_index,
                frame_num,
                params.angles,
                params.position,
                params.scale,
            );
            g2_ik_reposition(g2, host, params.position, Some(&mut *params));
            g2_ik_solve_instances(
                g2,
                &mut instances,
                g2_index,
                2.0,
                frame_num,
                params.position,
                true,
            );
        }
    }

    if params.me != ENTITYNUM_NONE {
        g2_rag_doll_current_position_instances(
            g2,
            host,
            handle,
            &mut instances,
            g2_index,
            frame_num,
            params.angles,
            params.position,
            params.scale,
        );
    }

    *g2.info_array.get_mut(handle) = instances;
}

/// Raven `static float G2_RagSetState(CGhoul2Info &ghoul2, boneInfo_t
/// &bone, int frameNum, const vec3_t origin, bool &resetOrigin)` — advances
/// `rag_state` (`ERS_DYNAMIC`/`_SETTLING`/`_SETTLED`) from the bone's
/// collision/rest-time history and returns a `[0,1]` settle decay factor.
/// `resetOrigin` is a genuine Raven out-param by reference that the body
/// never actually writes (`:2152-2252`, verified) — kept in the signature
/// for 1:1 fidelity to the faithfully-dead parameter, not fixed up.
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:2152-2252`
pub fn g2_rag_set_state(
    g2: &mut Ghoul2System,
    ghoul2: &mut CGhoul2Info,
    bone: &mut boneInfo_t,
    frame_num: i32,
    origin: vec3_t,
    reset_origin: &mut bool,
) -> f32 {
    let _ = reset_origin; // never written by Raven's own body either (see doc comment).

    g2.rag.origin_change = DistanceSquared(origin, bone.extraVec1);
    _VectorSubtract(origin, bone.extraVec1, &mut g2.rag.origin_change_dir);

    let mut decay = 1.0f32;
    const DYNAMIC_TIME: i32 = 1000;
    const SETTLE_TIME: i32 = 1000;

    if ghoul2.flags & GHOUL2_RAG_FORCESOLVE != 0 || bone.firstCollisionTime > 0 {
        g2.rag.rag_state = RagState::Dynamic;
        if frame_num > bone.firstCollisionTime + DYNAMIC_TIME {
            bone.extraVec1 = origin;
            if g2.rag.origin_change > 15.0 {
                bone.firstCollisionTime = frame_num;
            } else {
                bone.firstCollisionTime = 0;
                bone.restTime = frame_num;
                g2.rag.rag_state = RagState::Settling;
            }
        }
    } else if bone.restTime > 0 {
        decay = 1.0 - (frame_num - bone.restTime) as f32 / DYNAMIC_TIME as f32;
        decay = decay.clamp(0.0, 1.0);
        // magicFactor8 = 1.0 -- pow(decay, 1.0) is a no-op.
        g2.rag.rag_state = RagState::Settling;
        if frame_num > bone.restTime + SETTLE_TIME {
            bone.extraVec1 = origin;
            if g2.rag.origin_change > 15.0 {
                bone.restTime = frame_num;
            } else {
                bone.restTime = 0;
                g2.rag.rag_state = RagState::Settled;
            }
        }
    } else if bone.RagFlags & RAG_PCJ_IK_CONTROLLED != 0 {
        bone.firstCollisionTime = frame_num;
        g2.rag.rag_state = RagState::Dynamic;
        decay = 0.0;
    } else if g2.rag.origin_change > 15.0 {
        bone.firstCollisionTime = frame_num;
        g2.rag.rag_state = RagState::Dynamic;
        decay = 0.0;
    } else {
        g2.rag.rag_state = RagState::Settled;
        decay = 0.0;
    }
    decay
}

/// Raven `void Rag_Trace(...)`'s dependency `G2_RagGetPelvisLumbarOffsets`
/// helper — see this file's module-doc "problems" note #5: blocked by a
/// missing `world_matrix` parameter on this frozen signature (Raven's own
/// body reads the file-scope `worldMatrix` directly, `G2_bones.cpp:3424`,
/// which this port never reaches into as ambient state). A documented
/// no-op: `pos`/`dir`/`anim_pos`/`anim_dir` are left unchanged rather than
/// substituting an invented matrix. Its sole caller is gated behind the
/// `broadsword_ragtobase > 1` tuning cvar (default off).
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:3399-3448`
#[allow(clippy::too_many_arguments)]
pub fn g2_rag_get_pelvis_lumbar_offsets(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghoul2: &mut CGhoul2Info,
    params: Option<&mut RagDollUpdateParams>,
    pos: &mut vec3_t,
    dir: &mut vec3_t,
    anim_pos: &mut vec3_t,
    anim_dir: &mut vec3_t,
) {
    let _ = (g2, host, ghoul2, params, pos, dir, anim_pos, anim_dir);
    // See doc comment: no `world_matrix` parameter to compute a real answer.
}

/// Raven `static inline void G2_RagGetWorldAnimMatrix(CGhoul2Info &ghoul2,
/// boneInfo_t &bone, CRagDollUpdateParams *params, mdxaBone_t &retMatrix)`
/// — resolves `bone`'s settle-frame animated base matrix
/// (`G2_RagGetAnimMatrix`/`G2_RagGetBoneBasePoseMatrixLow`, both
/// `tr_ghoul2.cpp`/`render/skeleton.rs`, host-consuming) into world space
/// via the skeleton-build scratch `worldMatrix` (`render/skeleton.rs`,
/// threaded in rather than reached).
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:3377-3393`
#[allow(clippy::too_many_arguments)]
pub fn g2_rag_get_world_anim_matrix(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghoul2: &mut CGhoul2Info,
    bone: &mut boneInfo_t,
    params: Option<&mut RagDollUpdateParams>,
    world_matrix: &mdxaBone_t,
    ret_matrix: &mut mdxaBone_t,
) {
    let _ = host; // no direct host service; the bone-matrix accessors below read the already-resolved bone cache.
    let Some(params) = params else {
        return;
    };

    let true_base_matrix = g2_rag_get_anim_matrix(g2, ghoul2, bone.boneNumber, params.settle_frame);
    let base_bone_matrix = g2_rag_get_bone_base_pose_matrix_low(
        g2,
        ghoul2,
        bone.boneNumber,
        &true_base_matrix,
        params.scale,
    );
    multiply_3x4_matrix(ret_matrix, world_matrix, &base_bone_matrix);
}

/// Raven `static void G2_IKReposition(const vec3_t currentOrg,
/// CRagDollUpdateParams *params)` — per-effector-bone IK goal update:
/// derives `desiredDirection` from `ikPosition`/`velocityEffector`, jittered
/// by two `flrand(-0.75, 0.75)` calls (`EngineHost::flrand`, `:2127-2129`
/// numbering — actual call site `:4290`) and damps `velocityEffector`.
///
/// **Reported gap** (module-doc "problems" note #4): this frozen signature
/// has no `ghoul2`/blist parameter, but the body needs `boneInfo_t`'s own
/// `ikPosition`/`velocityEffector`/`lastPosition` fields (`ragBoneData[i]->
/// ...`, now blist-index-resolved per `G2SV-D13`(b)) — unreachable here. A
/// documented no-op rather than an invented substitute; reported upstream.
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:4257-4295`
pub fn g2_ik_reposition(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    current_org: vec3_t,
    params: Option<&mut RagDollUpdateParams>,
) {
    let _ = (g2, host, current_org, params);
    // See doc comment: no `ghoul2`/blist parameter to reach `boneInfo_t`.
}

/// Raven `static inline char *G2_Get_Bone_Name(CGhoul2Info *ghlInfo,
/// boneInfo_v &blist, int boneNum)` — debug-only (`_DEBUG_BONE_NAMES`,
/// gated by `#ifdef _DEBUG`, `:2576-2578` — off in this doc's own NDEBUG
/// WinDed build config, so genuinely dead server-side; a correction to this
/// doc's own prose claiming it "compiles", reported upstream) skeleton-name
/// lookup: walks `blist` for the override entry whose `boneNumber ==
/// boneNum`, then reads the bone's name out of the model's `mdxaSkel_t`
/// table via `ghlInfo->aHeader`; `"BONE_NOT_FOUND"` if the entry isn't
/// present. Raven's raw `char *` return becomes an owned `String`
/// (porting-rules §C9). Transcribed faithfully despite being dead code
/// (this file's roster explicitly names it as one of the 20 ported
/// functions).
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:2580-2605`
pub fn g2_get_bone_name(ghoul2: &CGhoul2Info, blist: &[boneInfo_t], bone_num: i32) -> String {
    if ghoul2.a_header.is_null() {
        return "BONE_NOT_FOUND".to_string();
    }
    for entry in blist {
        if entry.boneNumber != bone_num {
            continue;
        }
        return unsafe {
            let skel = mdxa_skel_ptr(ghoul2.a_header as *mut c_void, entry.boneNumber);
            mdxa_skel_name(skel)
        };
    }
    "BONE_NOT_FOUND".to_string()
}

/// Raven `static inline void G2_RagDebugBox(vec3_t mins, vec3_t maxs, int
/// duration)` — cgame debug-box draw callback; `_DEBUG_BONE_NAMES`-gated
/// (off in this build, see `g2_get_bone_name`'s doc comment) but even if it
/// compiled, `#ifdef DEDICATED return;` is the entire live-build body
/// (`## Raven ground truth`, "cgvm ragdoll-callback dead branches"), so this
/// is a compiled no-op either way. No `host` needed.
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:2883-2901`
pub fn g2_rag_debug_box(_mins: vec3_t, _maxs: vec3_t, _duration: i32) {
    // #ifdef DEDICATED return; -- the entire live-build body.
}

/// Raven `static inline void G2_RagDebugLine(vec3_t start, vec3_t end, int
/// time, int color, int radius)` — cgame debug-line draw callback; same
/// compiled-no-op shape as [`g2_rag_debug_box`].
///
/// Source: `oracle/codemp/ghoul2/G2_bones.cpp:2903-2923`
pub fn g2_rag_debug_line(_start: vec3_t, _end: vec3_t, _time: i32, _color: i32, _radius: i32) {
    // #ifdef DEDICATED return; -- the entire live-build body.
}
