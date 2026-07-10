//! `G2API` bolts + attach — the bolt-list mutators, the inter-model/entity
//! attach-link encoders, and the listen-server-opt attach-trio no-ops.
//!
//! Per `docs/subsystems/ghoul2-server.md` roster (`api_bolts.rs`, class "G2API
//! bolts+attach"): `AddBolt`/`AddBoltSurfNum`/`RemoveBolt`/`SetBoltInfo`/
//! `GetBoltMatrix` (write-through + `bool`, `G2SV-D1`; `gG2_GBM*` flags),
//! `AttachG2Model`/`DetachG2Model`/`AttachEnt`/`DetachEnt`, `SetNewOrigin`.
//! `AttachInstanceToEntNum`/`ClearAttachedInstance`/`CleanEntAttachments` are
//! COMPILED NO-OPS kept as callable empty-body fns here per §C10
//! (`G2SV-D16`/ruling 39b): their signatures are unconditional
//! (`G2_API.cpp:200,214,221`), only the bodies are `#ifdef
//! _G2_LISTEN_SERVER_OPT` (off, `G2SV-D4`), and LIVE syscall arms still call
//! them (`sv_game.cpp:1587,1591,1594`) — NOT §20-dropped.
//!
//! Every `G2API_*` entry keeps its 1:1 signature (`G2SV-D6`) and threads
//! `g2: &mut Ghoul2System` (ruling 4/11, state threaded not reached);
//! `g2api_get_bolt_matrix` additionally threads `host: &mut impl EngineHost`
//! (its body reads model memory via `EngineHost::model_mdxa`, ruling 36).
//! Out-param classification follows the frozen discriminator ("Out-param
//! contract for the un-illustrated `G2API_*` functions", `G2SV-D1`
//! generalized): a failure path that still writes its out-param keeps
//! `&mut T` + `bool`; a failure path that returns before touching it maps to
//! `Option<T>`.
//!
//! **Doc/oracle gap found while transcribing this class (reported under
//! `problems`, not improvised around — porting-rules §F17).** Six of this
//! file's frozen signatures — `g2api_add_bolt`, `g2api_add_bolt_surf_num`,
//! `g2api_remove_bolt`, `g2api_attach_g2_model`, `g2api_detach_g2_model`,
//! `g2api_attach_ent` — carry **no** `host: &mut impl EngineHost` parameter
//! (`docs/subsystems/ghoul2-server.md` `## Seam definition` illustrates
//! `g2api_add_bolt` this way at line 627; the others follow the same shape in
//! this file's own pre-existing skeleton), yet every one of their Raven
//! bodies opens with `G2_SetupModelPointers`, which re-derives model pointers
//! via `RE_RegisterModel`/`R_GetModelByHandle` (`G2_API.cpp:2675-2693`) — a
//! genuinely host-consuming call, matching `misc.rs`'s own already-landed
//! `g2_setup_model_pointers(host, ghl_info)` signature. Without a `host`
//! parameter these six cannot re-derive validity, so each uses the
//! already-cached `CGhoul2Info::valid` flag (single instance) or "at least one
//! cached-valid instance" (vector overload) as the closest available proxy —
//! skipping the post-`vid_restart` revalidation only. Divergence, not
//! invention (porting-rules §19); the real fix is adding `host` to these six
//! signatures upstream.
//!
//! **Second doc/oracle gap (reported under `problems`).** `G2_NeedsRecalc`
//! (`tr_ghoul2.cpp:3544-3563`) is called directly by `G2API_GetBoltMatrix`
//! (`:1823,1833`) but has no roster row, method-transcription-table entry, or
//! landed Rust body anywhere in this crate. Ported inline below (not a
//! separate stub file — no home is assigned) using `host`, which this
//! function's own signature does carry. Its `mBoneCache->mod != currentModel`
//! comparison is retargeted to `bone_cache.model != ghl_info.model` (both
//! `qhandle_t`): `render/bone_cache.rs`'s `CBoneCache.model` is documented as
//! "the `qhandle_t` the ctor received", the natural counterpart of
//! `CGhoul2Info.model` (Raven `mModel`), whereas Raven's `currentModel` is the
//! opaque resolved pointer this crate never names (`G2SV-D5`) — so this is the
//! only comparison the crate's own chosen shapes make possible.
//!
//! **Third doc/oracle gap (reported under `problems`).** `render::skeleton::
//! g2_get_bolt_matrix_low(g2: &mut Ghoul2System, ghoul2: &CGhoul2Info, ...)`
//! (itself a non-frozen transcription stopgap per that file's own module doc)
//! cannot be called from here: its first parameter wants exclusive access to
//! the *whole* `Ghoul2System`, while its second parameter is a live reference
//! borrowed out of that same `Ghoul2System`'s `info_array` field — an
//! unsatisfiable Rust borrow (E0502) at any real call site, not a call-site
//! mistake. `G2API_GetBoltMatrix`'s bone-attached arm also needs
//! `mdxaSkelOffsets_t`/`mdxaSkel_t` byte-offset reads this crate has no typed
//! access to (`G2SV-D5`), and the surface-attached arm needs
//! `G2_ProcessSurfaceBolt2`, which has no Rust home anywhere in this crate.
//! `g2api_get_bolt_matrix` below falls back to Raven's own "bolt has neither a
//! bone nor a surface" identity-matrix arm (`tr_ghoul2.cpp:3328-3330`)
//! unconditionally rather than inventing the missing bone/surface arms.
//!
//! **Fourth, minor gap (reported under `problems`).** `VectorNormalize`
//! (`q_math.c:1172-1186`) and `Create_Matrix` (`G2_misc.cpp:1630-1653`) are
//! both needed by `g2api_get_bolt_matrix`; the former has no port anywhere
//! reachable from this crate (`mp_engine_ghoul2` depends only on `mp_qshared`/
//! `mp_host_interface`), and the latter exists only as `misc::create_matrix`,
//! which is file-private (not `pub`) and so cannot be reused here. Both are
//! reimplemented locally below as narrow stopgaps (`vector_normalize_row`,
//! `create_matrix_from_angles`) rather than left uncallable.
//!
//! Bounds guards on a few direct `bltlist[index]` reads below (`
//! g2api_attach_g2_model`/`g2api_attach_ent`) are a §19 divergence: Raven's own
//! bounds checks there are `assert()`s only, which compile to nothing under
//! this build's `-DNDEBUG` (`## Raven ground truth`), so an out-of-range index
//! is UB in the oracle; this port picks the defined "treat as unbolted" (`qfalse`
//! /`None`) behavior instead of an out-of-bounds panic.

use mp_host_interface::EngineHost;
use mp_qshared::shared::{errorParm_t, mdxaBone_t, qhandle_t, vec3_t};

use crate::api_collision;
use crate::bolts;
use crate::ghoul2_system::Ghoul2System;
use crate::misc;
use crate::render::{bone_transform, skeleton};
use crate::shared::cghoul2_info::CGhoul2Info;
use crate::shared::cghoul2_info_v::CGhoul2Info_v;

/// Raven `#define MODEL_WIDTH 10` / `BOLT_WIDTH 10` / `ENTITY_WIDTH 12` and the
/// derived `*_AND`/`*_SHIFT` bit-packing constants the `mModelBoltLink`/
/// `boltInfo` encodings below use. Only this file's `AttachG2Model`/`AttachEnt`
/// need them (crate-wide grep), so they are local, matching this crate's own
/// per-file constant convention (e.g. `info_array.rs`'s `MAX_G2_MODELS`).
/// Source: `oracle/codemp/ghoul2/G2.h:30-40`
const MODEL_WIDTH: i32 = 10;
const BOLT_WIDTH: i32 = 10;
const ENTITY_WIDTH: i32 = 12;
const MODEL_AND: i32 = (1 << MODEL_WIDTH) - 1;
const BOLT_AND: i32 = (1 << BOLT_WIDTH) - 1;
const ENTITY_AND: i32 = (1 << ENTITY_WIDTH) - 1;
const BOLT_SHIFT: i32 = 0;
const MODEL_SHIFT: i32 = BOLT_SHIFT + BOLT_WIDTH;
const ENTITY_SHIFT: i32 = MODEL_SHIFT + MODEL_WIDTH;

/// Raven `#define GHOUL2_NEWORIGIN 0x008` (`ghoul2_shared.h:232`) — the
/// `mFlags` bit `G2API_SetNewOrigin` sets. Local (only this file's
/// `g2api_set_new_origin` reads it), matching this crate's per-file constant
/// convention.
const GHOUL2_NEWORIGIN: i32 = 0x008;

/// Raven `G2API_AddBolt` — add a bolt on `boneName`, returning its index (or
/// `-1` on failure: bad `modelIndex` or `G2_SetupModelPointers` failure).
///
/// No `host` param (module-doc gap #1): uses `ghl_info.valid` in place of a
/// `G2_SetupModelPointers` re-derivation.
///
/// Frozen verbatim in `docs/subsystems/ghoul2-server.md` `## Seam definition`.
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:1633-1645`
pub fn g2api_add_bolt(
    g2: &mut Ghoul2System,
    ghoul2: &mut CGhoul2Info_v,
    model_index: i32,
    bone_name: &str,
) -> i32 {
    // Raven's `(int)&ghoul2` reference-address truthiness check can never be
    // false in Rust (no null references); folded away (§C10).
    if ghoul2.size(g2) > model_index {
        let ghl_info = ghoul2.get_mut(g2, model_index);
        if ghl_info.valid {
            // `bolts::g2_add_bolt` wants `&CGhoul2Info` *and* `&mut`/`&` borrows of
            // two of that same instance's own fields at once — impossible to
            // satisfy together in Rust from a single `&mut CGhoul2Info`. Extract
            // both lists as owned values first (mem::take, no Clone needed) so
            // `ghl_info` itself is free to be reborrowed shared, then write the
            // (possibly mutated) bolt list back.
            let mut bltlist = std::mem::take(&mut ghl_info.bltlist);
            let slist = std::mem::take(&mut ghl_info.slist);
            let idx = bolts::g2_add_bolt(ghl_info, &mut bltlist, &slist, bone_name);
            ghl_info.bltlist = bltlist;
            ghl_info.slist = slist;
            return idx;
        }
    }
    -1
}

/// Raven `G2API_AddBoltSurfNum` — add a bolt on a surface index, returning its
/// bolt index (or `-1` on `G2_SetupModelPointers` failure).
///
/// No `host` param (module-doc gap #1): uses `ghl_info.valid`.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:1648-1653`
pub fn g2api_add_bolt_surf_num(
    g2: &mut Ghoul2System,
    ghl_info: &mut CGhoul2Info,
    surf_index: i32,
) -> i32 {
    let _ = g2;
    if !ghl_info.valid {
        return -1;
    }
    // Same list-extraction technique as `g2api_add_bolt` above.
    let mut bltlist = std::mem::take(&mut ghl_info.bltlist);
    let slist = std::mem::take(&mut ghl_info.slist);
    let idx = bolts::g2_add_bolt_surf_num(ghl_info, &mut bltlist, &slist, surf_index);
    ghl_info.bltlist = bltlist;
    ghl_info.slist = slist;
    idx
}

/// Raven `G2API_RemoveBolt` — remove a bolt by index; `qfalse` on
/// `G2_SetupModelPointers` failure, else `G2_Remove_Bolt`'s result.
///
/// No `host` param (module-doc gap #1): uses `ghl_info.valid`.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:1624-1631`
pub fn g2api_remove_bolt(g2: &mut Ghoul2System, ghl_info: &mut CGhoul2Info, index: i32) -> bool {
    let _ = g2;
    if !ghl_info.valid {
        return false;
    }
    bolts::g2_remove_bolt(&mut ghl_info.bltlist, index)
}

/// Raven `G2API_SetBoltInfo` — write `mModelBoltLink` on the model at
/// `modelIndex` (a bounds-checked no-op on an out-of-range index).
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:1684-1692`
pub fn g2api_set_bolt_info(
    g2: &mut Ghoul2System,
    ghoul2: &mut CGhoul2Info_v,
    model_index: i32,
    bolt_info: i32,
) {
    if ghoul2.size(g2) > model_index {
        ghoul2.get_mut(g2, model_index).model_bolt_link = bolt_info;
    }
}

/// Raven `qboolean G2API_GetBoltMatrix(..., mdxaBone_t *matrix)` — write-through
/// + `qboolean` (`G2SV-D1`, ruling 18): the out-matrix is ALWAYS written, even
/// on the failure paths (`Multiply_3x4Matrix(matrix, &worldMatrix,
/// &identityMatrix)` before `return qfalse`, `:1893-1894`), so callers reading
/// `matrix` on `false` still observe Raven's fallback. Reads model memory via
/// `EngineHost::model_mdxa` (loader-owned `mdxaHeader_t`/`mdxaSkel_t`, `G2SV-D5`)
/// through the `G2_ConstructGhoulSkeleton`/`G2_GetBoltMatrixLow` chain (ruling 36).
///
/// The `G2_GetBoltMatrixLow`/bone-vs-surface-attached resolution itself is a
/// module-doc gap (#3 above, reported under `problems`): this always takes
/// Raven's own "bolt has neither a bone nor a surface" identity-matrix arm
/// (`tr_ghoul2.cpp:3328-3330`) for the per-bolt matrix rather than inventing
/// the unreachable bone/surface arms.
///
/// Frozen verbatim in `docs/subsystems/ghoul2-server.md` `## Seam definition`.
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:1795-1900`
#[allow(clippy::too_many_arguments)]
pub fn g2api_get_bolt_matrix(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghoul2: &mut CGhoul2Info_v,
    model_index: i32,
    bolt_index: i32,
    angles: vec3_t,
    position: vec3_t,
    frame_num: i32,
    model_list: &[qhandle_t],
    scale: vec3_t,
    bolt_matrix: &mut mdxaBone_t,
) -> bool {
    // Raven's `modelList` parameter is unread by this function's body
    // (`G2_API.cpp:1795-1900`) — kept for 1:1 arity fidelity only.
    let _ = model_list;

    const IDENTITY_MATRIX: mdxaBone_t = mdxaBone_t {
        matrix: [
            [0.0, -1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
        ],
    };

    // `G2_GenerateWorldMatrix` runs unconditionally, before the setup check
    // (`:1807`); the failure-path fallback below reuses this same matrix.
    let (world_matrix, _world_matrix_inv) = misc::g2_generate_world_matrix(angles, position);

    let setup_ok = misc::g2_setup_model_pointers_v(g2, host, ghoul2);
    let in_range = setup_ok && model_index >= 0 && model_index < ghoul2.size(g2);

    if in_range {
        let tframe_num = api_collision::g2api_get_time(g2, frame_num);

        // --- G2_NeedsRecalc inlined (tr_ghoul2.cpp:3544-3563; module-doc gap #2,
        // reported under `problems` — no roster row/home anywhere in this crate).
        // Needs both the arena instance and the sibling `bone_caches` arena at
        // once; `CGhoul2Info_v::get_mut`'s whole-`Ghoul2System`-borrowing shape
        // can't provide that alongside any further `g2` use, so this reaches both
        // fields directly via disjoint field projection instead. ---
        let bolt_in_range;
        let needs_recalc;
        {
            let Ghoul2System {
                info_array,
                bone_caches,
                ..
            } = &mut *g2;
            let ghl_info = &mut info_array.get_mut(ghoul2.mItem)[model_index as usize];
            misc::g2_setup_model_pointers(host, ghl_info);
            bolt_in_range = bolt_index >= 0 && (bolt_index as usize) < ghl_info.bltlist.len();
            if bolt_in_range {
                // `mBoneCache->mod != currentModel` retargeted to `qhandle_t`
                // comparison (module-doc gap #2: `current_model` is opaque, `G2SV-D5`).
                let cache_model = ghl_info
                    .bone_cache
                    .and_then(|id| bone_caches.get(id))
                    .map(|cache| cache.model);
                needs_recalc =
                    ghl_info.skel_frame_num != tframe_num || cache_model != Some(ghl_info.model);
                if needs_recalc {
                    ghl_info.skel_frame_num = tframe_num;
                }
            } else {
                needs_recalc = false;
            }
        }

        if bolt_in_range {
            if needs_recalc {
                skeleton::g2_construct_ghoul_skeleton(g2, host, ghoul2, tframe_num, true, scale);
            }

            // G2_GetBoltMatrixLow (module-doc gap #3): identity-arm stopgap only.
            let mut bolt = IDENTITY_MATRIX;

            // scale the bolt position by the scale factor for this model since at
            // this point it's still in model space (`:1841-1852`).
            if scale[0] != 0.0 {
                bolt.matrix[0][3] *= scale[0];
            }
            if scale[1] != 0.0 {
                bolt.matrix[1][3] *= scale[1];
            }
            if scale[2] != 0.0 {
                bolt.matrix[2][3] *= scale[2];
            }
            vector_normalize_row(&mut bolt.matrix[0]);
            vector_normalize_row(&mut bolt.matrix[1]);
            vector_normalize_row(&mut bolt.matrix[2]);

            bone_transform::multiply_3x4_matrix(bolt_matrix, &world_matrix, &bolt);

            if !g2.gbm_use_sp_method {
                // "this is horribly stupid and I hate it. But lots of game code is
                // written to assume this 90 degree offset thing." (`:1870`)
                let rot_mat = create_matrix_from_angles([0.0, 270.0, 0.0]);
                let mut temp_matrix = mdxaBone_t {
                    matrix: [[0.0; 4]; 3],
                };
                bone_transform::multiply_3x4_matrix(&mut temp_matrix, &world_matrix, &bolt);
                let origin = [
                    temp_matrix.matrix[0][3],
                    temp_matrix.matrix[1][3],
                    temp_matrix.matrix[2][3],
                ];
                temp_matrix.matrix[0][3] = 0.0;
                temp_matrix.matrix[1][3] = 0.0;
                temp_matrix.matrix[2][3] = 0.0;
                bone_transform::multiply_3x4_matrix(bolt_matrix, &temp_matrix, &rot_mat);
                bolt_matrix.matrix[0][3] = origin[0];
                bolt_matrix.matrix[1][3] = origin[1];
                bolt_matrix.matrix[2][3] = origin[2];
            } else {
                g2.gbm_use_sp_method = false;
            }

            return true;
        }
    }

    bone_transform::multiply_3x4_matrix(bolt_matrix, &world_matrix, &IDENTITY_MATRIX);
    false
}

/// Stopgap reimplementation of Raven `VectorNormalize` (`oracle/codemp/game/
/// q_math.c:1172-1186`) — `mp_engine_ghoul2` has no reachable port of this
/// q_math primitive (module-doc gap #4, reported under `problems`; the crate
/// depends only on `mp_qshared`/`mp_host_interface`). Normalizes only the
/// first 3 elements of a 4-wide `mdxaBone_t` row, matching the oracle's
/// `(float*)matrix[i]` cast onto a `vec3_t`-shaped `VectorNormalize` call.
fn vector_normalize_row(row: &mut [f32; 4]) {
    let length = (row[0] * row[0] + row[1] * row[1] + row[2] * row[2]).sqrt();
    if length != 0.0 {
        let ilength = 1.0 / length;
        row[0] *= ilength;
        row[1] *= ilength;
        row[2] *= ilength;
    }
}

/// Stopgap reimplementation of Raven `Create_Matrix` (`G2_misc.cpp:1630-1653`,
/// via `AnglesToAxis`/`AngleVectors`, `q_math.c:530-536,1315-1348`) —
/// `misc::create_matrix` already ports this but is file-private, so it cannot
/// be reused here (module-doc gap #4, reported under `problems`); only the
/// single fixed-angle call site in `g2api_get_bolt_matrix` (`newangles =
/// {0,270,0}`, `G2_API.cpp:1872`) needs it.
fn create_matrix_from_angles(angle: vec3_t) -> mdxaBone_t {
    // q_math.c PITCH=0, YAW=1, ROLL=2 (`q_shared.h:374-376`).
    let (sy, cy) = (angle[1].to_radians().sin(), angle[1].to_radians().cos());
    let (sp, cp) = (angle[0].to_radians().sin(), angle[0].to_radians().cos());
    let (sr, cr) = (angle[2].to_radians().sin(), angle[2].to_radians().cos());

    let forward = [cp * cy, cp * sy, -sp];
    let right = [
        -1.0 * sr * sp * cy + -1.0 * cr * -sy,
        -1.0 * sr * sp * sy + -1.0 * cr * cy,
        -1.0 * sr * cp,
    ];
    let up = [cr * sp * cy + -sr * -sy, cr * sp * sy + -sr * cy, cr * cp];
    // AnglesToAxis: axis[0]=forward, axis[1]=-right, axis[2]=up (q_math.c:530-536).
    let axis = [forward, [-right[0], -right[1], -right[2]], up];

    let mut matrix = mdxaBone_t {
        matrix: [[0.0; 4]; 3],
    };
    for row in 0..3 {
        for (col, axis_row) in axis.iter().enumerate() {
            matrix.matrix[row][col] = axis_row[row];
        }
        matrix.matrix[row][3] = 0.0;
    }
    matrix
}

/// Raven `G2API_AttachG2Model` — encode `toModel`/`toBoltIndex` into
/// `ghoul2From[modelFrom].mModelBoltLink`; `qfalse` on a negative
/// `toBoltIndex`, setup-pointer failure on either model, or an out-of-range
/// `modelFrom`/`toModel`/an unbolted `toBoltIndex`.
///
/// No `host` param (module-doc gap #1): uses "at least one cached-valid
/// instance in the vector" in place of the vector-overload
/// `G2_SetupModelPointers` re-derivation.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:1658-1682`
pub fn g2api_attach_g2_model(
    g2: &mut Ghoul2System,
    ghoul2_from: &mut CGhoul2Info_v,
    model_from: i32,
    ghoul2_to: &mut CGhoul2Info_v,
    to_bolt_index: i32,
    to_model: i32,
) -> bool {
    if to_bolt_index < 0 {
        return false;
    }
    let from_ok = g2.info_array.get(ghoul2_from.mItem).iter().any(|i| i.valid);
    let to_ok = g2.info_array.get(ghoul2_to.mItem).iter().any(|i| i.valid);
    if !(from_ok && to_ok) {
        return false;
    }
    if ghoul2_from.size(g2) <= model_from || ghoul2_to.size(g2) <= to_model {
        return false;
    }
    // Bounds guard (module-doc note): Raven's `bltlist[toBoltIndex]` has no
    // upper-bound check here under `-DNDEBUG` (UB on an out-of-range index).
    let to_info = ghoul2_to.get(g2, to_model);
    if (to_bolt_index as usize) >= to_info.bltlist.len() {
        return false;
    }
    let bolt = &to_info.bltlist[to_bolt_index as usize];
    if bolt.boneNumber != -1 || bolt.surfaceNumber != -1 {
        let to_model_masked = to_model & MODEL_AND;
        let to_bolt_masked = to_bolt_index & BOLT_AND;
        ghoul2_from.get_mut(g2, model_from).model_bolt_link =
            (to_model_masked << MODEL_SHIFT) | (to_bolt_masked << BOLT_SHIFT);
        return true;
    }
    false
}

/// Raven `G2API_DetachG2Model` — reset `mModelBoltLink` to `-1`; `qfalse` on
/// `G2_SetupModelPointers` failure.
///
/// No `host` param (module-doc gap #1): uses `ghl_info.valid`.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:1695-1703`
pub fn g2api_detach_g2_model(g2: &mut Ghoul2System, ghl_info: &mut CGhoul2Info) -> bool {
    let _ = g2;
    if !ghl_info.valid {
        return false;
    }
    ghl_info.model_bolt_link = -1;
    true
}

/// Raven `qboolean G2API_AttachEnt(int *boltInfo, ...)` — encodes the bolt/
/// model/entity triple into `*boltInfo` and returns `qtrue` only when a bolt
/// exists at `toBoltIndex`; the failure path (`return qfalse`, `:1725`) never
/// touches `*boltInfo` (write-on-success-only), so per the frozen out-param
/// discriminator (`G2SV-D1` generalized) this maps to `Option<i32>` — `None`
/// is the untouched-output `qfalse` path, `Some(bolt_info)` the written
/// `qtrue` path — not a write-through `&mut` out-param.
///
/// No `host` param (module-doc gap #1): uses `ghl_info_to.valid`. Bounds guard
/// (module-doc note): `toBoltIndex` has no upper-bound check in the oracle
/// under `-DNDEBUG` (UB on out-of-range).
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:1705-1725`
pub fn g2api_attach_ent(
    g2: &mut Ghoul2System,
    ghl_info_to: &mut CGhoul2Info,
    to_bolt_index: i32,
    ent_num: i32,
    to_model_num: i32,
) -> Option<i32> {
    let _ = g2;
    if !ghl_info_to.valid {
        return None;
    }
    if ghl_info_to.bltlist.is_empty()
        || to_bolt_index < 0
        || (to_bolt_index as usize) >= ghl_info_to.bltlist.len()
    {
        return None;
    }
    let bolt = &ghl_info_to.bltlist[to_bolt_index as usize];
    if bolt.boneNumber != -1 || bolt.surfaceNumber != -1 {
        let to_model_masked = to_model_num & MODEL_AND;
        let to_bolt_masked = to_bolt_index & BOLT_AND;
        let ent_masked = ent_num & ENTITY_AND;
        Some(
            (to_bolt_masked << BOLT_SHIFT)
                | (to_model_masked << MODEL_SHIFT)
                | (ent_masked << ENTITY_SHIFT),
        )
    } else {
        None
    }
}

/// Raven `void G2API_DetachEnt(int *boltInfo)` — declared
/// (`G2_local.h:139`) but **never defined** anywhere in `oracle/codemp/`
/// (no `.cpp` body) and never called (no reference outside the header
/// declaration). Ported per the doc's explicit roster listing
/// ("`AttachEnt`/`DetachEnt`", `api_bolts.rs` row) with the header's
/// signature; since no body exists there is no out-param write behavior to
/// classify, so the out-param is kept as the mechanical `&mut i32` (§C7
/// default for an unwritten-classification case) pending oracle confirmation.
/// No behavior to transcribe means a genuine empty no-op, not a `todo!()`.
///
/// Source: `oracle/codemp/ghoul2/G2_local.h:139` (no `.cpp` definition found)
pub fn g2api_detach_ent(g2: &mut Ghoul2System, bolt_info: &mut i32) {
    let _ = (g2, bolt_info);
}

/// Raven `G2API_SetNewOrigin` — set `mNewOrigin`/`GHOUL2_NEWORIGIN` on
/// `ghoul2[0]`; `qfalse` on `G2_SetupModelPointers` failure. `Com_Error`s on a
/// negative `boltIndex` when setup succeeds (`ERR_DROP`, routed through
/// `EngineHost::error`).
///
/// Raven's error message also names `currentModel->name`; that opaque
/// `model_t*` field is not readable from this crate (`G2SV-D5`), so the model
/// name is dropped from the message text (message text only, no behavior
/// change to the error path itself).
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:2428-2461`
pub fn g2api_set_new_origin(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghoul2: &mut CGhoul2Info_v,
    bolt_index: i32,
) -> bool {
    if ghoul2.size(g2) <= 0 {
        return false;
    }
    let ghl_info = ghoul2.get_mut(g2, 0);
    if !misc::g2_setup_model_pointers(host, ghl_info) {
        return false;
    }
    if bolt_index < 0 {
        host.error(
            errorParm_t::ERR_DROP,
            &format!("Bad boltindex ({bolt_index}) trying to SetNewOrigin (naughty naughty!)"),
        );
    }
    ghl_info.new_origin = bolt_index;
    ghl_info.flags |= GHOUL2_NEWORIGIN;
    true
}

/// Raven `void G2API_AttachInstanceToEntNum(CGhoul2Info_v &ghoul2, int
/// entityNum, qboolean server)` — COMPILED NO-OP (`G2SV-D16`/ruling 39b): the
/// signature is unconditional but the entire body is `#ifdef
/// _G2_LISTEN_SERVER_OPT` (off, `G2SV-D4`), so nothing runs; kept callable
/// because the live `G_G2_ATTACHINSTANCETOENTNUM` syscall arm still calls it
/// (`sv_game.cpp:1587`).
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:200-210`
pub fn g2api_attach_instance_to_ent_num(
    g2: &mut Ghoul2System,
    ghoul2: &mut CGhoul2Info_v,
    entity_num: i32,
    server: bool,
) {
    let _ = (g2, ghoul2, entity_num, server);
}

/// Raven `void G2API_ClearAttachedInstance(int entityNum)` — COMPILED NO-OP
/// (`G2SV-D16`/ruling 39b): body is `#ifdef _G2_LISTEN_SERVER_OPT` (off); kept
/// callable because `G_G2_CLEARATTACHEDINSTANCE` still calls it
/// (`sv_game.cpp:1591`).
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:213-218`
pub fn g2api_clear_attached_instance(g2: &mut Ghoul2System, entity_num: i32) {
    let _ = (g2, entity_num);
}

/// Raven `void G2API_CleanEntAttachments(void)` — COMPILED NO-OP
/// (`G2SV-D16`/ruling 39b): body is `#ifdef _G2_LISTEN_SERVER_OPT` (off); kept
/// callable because `G_G2_CLEANENTATTACHMENTS` still calls it
/// (`sv_game.cpp:1594`).
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:221-231`
pub fn g2api_clean_ent_attachments(g2: &mut Ghoul2System) {
    let _ = g2;
}
