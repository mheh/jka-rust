//! `G2_TransformBone` — the welded renderer bone-evaluation free-function
//! chain (`docs/subsystems/ghoul2-server.md` roster, `render/bone_transform.rs`,
//! class "G2_TransformBone"): "`G2_TransformBone`, `Multiply_3x4Matrix`,
//! `G2_CreateQuaterion`, `G2_CreateMatrixFromQuaterion` (`-ffp-contract=off`,
//! `G2SV-D6`). `Inverse_Matrix` is in `misc.rs` (`G2_misc.cpp:1656`), not here."
//!
//! These are pure bone math: no `Ghoul2System`/`EngineHost` threading (the
//! frozen `## Seam definition` signatures for `g2_transform_bone` and
//! `multiply_3x4_matrix` take no `g2`/`host` params — `CBoneCache` already
//! holds everything `G2_TransformBone` reads, and the matrix helpers are
//! scratch-buffer math). `G2_TransformBone` is called by `CBoneCache::EvalLow`
//! (`render/bone_cache.rs`, `tr_ghoul2.cpp:236,1541`); `Multiply_3x4Matrix` and
//! the quaternion helpers are the LERP chain it drives.
//!
//! **Finding reported upstream (see the porting task's `problems` output, kept
//! out of the frozen doc per house rules):** `G2_TimingModel`
//! (`tr_ghoul2.cpp:1167-1407`) is a private helper called from
//! `G2_TransformBone`'s body (`:1596`, this file's domain) **and** separately
//! from `G2_bones.cpp:885` (`bones.rs`'s domain, non-ragdoll bone logic) — it
//! is not named in either roster row's summary. `bones.rs` (landed) already
//! attributes its home to `render/bone_cache.rs` in a comment
//! (`g2_get_bone_anim_index`, citing `tr_ghoul2.cpp:1167`), not to this file;
//! the skeleton left it unstubbed here to avoid a duplicate/conflicting
//! *public* definition. But `g2_transform_bone`'s own body (`:1596`) reaches
//! it directly and `render/bone_cache.rs` has no landed function for it
//! either (still `todo!()`), so — mirroring the exact precedent `bones.rs`
//! itself already set for this identical problem (its own module-doc finding
//! 4) — [`g2_timing_model`] below is a third, file-local, faithful duplicate
//! (mechanical transcription of `tr_ghoul2.cpp:1167-1407`, not invented
//! behavior), `pub(self)` and used only by [`g2_transform_bone`]. Unlike
//! `bones.rs`'s copy (forced to take `&boneInfo_t` by its own call site's
//! pinned read-only `blist` signature — its module-doc finding 5), this
//! file's `boneList` comes off `CBoneCache::root_bone_list` (a raw `*mut
//! Vec<boneInfo_t>`), so this copy takes `&mut boneInfo_t` and performs
//! Raven's one write (`bone.flags &= ~(BONE_ANIM_TOTAL)`) faithfully — no
//! signature-forced divergence here. Whichever porter lands
//! `render/bone_cache.rs`'s real home for `G2_TimingModel` should reconcile
//! all three copies.
//!
//! **Second finding reported upstream (same reason):** `UnCompressBone`
//! (`tr_ghoul2.cpp:1158-1163`, `/*static inline*/`) and its private helper
//! `G2_GetBonePoolIndex` (`:1148-1155`, `static`) are neither named in this
//! file's roster/method-table row nor in any other roster row, yet
//! `UnCompressBone` is called five times directly inside `G2_TransformBone`'s
//! body (`:1707,1708,1723,1747,1748`, within this row's own cited
//! `1541-2051` extent) and once more from `G2_RagGetAnimMatrix`
//! (`:1464`, the not-yet-landed `ragdoll.rs`'s domain). `matcomp.rs`
//! (landed) already anchors `MC_UnCompressQuat`'s "sole live consumer" as
//! `UnCompressBone` "inside the frozen bone-eval chain" — i.e. this file —
//! so both are stubbed here (the primary, 5-call home) rather than left
//! unstubbed like `G2_TimingModel`, whose other home is already landed
//! elsewhere; a missing stub here would block both this file's own
//! `g2_transform_bone` and the future `ragdoll.rs` porter.
//!
//! `G2_Find_Bone_In_List` (`:1559`) is also called from this file's cited
//! range but is already landed in `bones.rs` as `g2_find_bone_in_list`
//! (`crates/mp/engine/ghoul2/src/bones.rs:340`) — not re-stubbed here.

use core::ffi::c_void;

use mp_qshared::shared::{mdxaBone_t, vec4_t, MAX_QPATH};

use crate::render::bone_cache::CBoneCache;
use crate::shared::bone_info_t::boneInfo_t;

// ---------------------------------------------------------------------------
// `boneInfo_t::flags` bit constants (`G2.h:8-26`). Duplicated locally, same
// convention `bones.rs` already established for this crate (no shared
// flags-constants module yet).
// ---------------------------------------------------------------------------

/// Source: `oracle/codemp/ghoul2/G2.h:8`
const BONE_ANGLES_PREMULT: i32 = 0x0001;
/// Source: `oracle/codemp/ghoul2/G2.h:9`
const BONE_ANGLES_POSTMULT: i32 = 0x0002;
/// Source: `oracle/codemp/ghoul2/G2.h:10`
const BONE_ANGLES_REPLACE: i32 = 0x0004;
/// Source: `oracle/codemp/ghoul2/G2.h:17`
const BONE_ANGLES_RAGDOLL: i32 = 0x2000;
/// Source: `oracle/codemp/ghoul2/G2.h:19`
const BONE_ANGLES_IK: i32 = 0x4000;
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

// ---------------------------------------------------------------------------
// `mdxaHeader_t`/`mdxaSkel_t` byte-offset helpers.
//
// `G2SV-D5` forbids naming `mdxaHeader_t`/`mdxaSkel_t`/`mdxaIndex_t`/
// `mdxaCompQuatBone_t` as Rust types in this crate. This is another
// file-local copy of the same byte-arithmetic `bones.rs`/`api_bones.rs`/
// `api_models.rs` already duplicate for the same header (reported upstream
// there, followed here for consistency, not a new decision).
//
// Source: `oracle/codemp/renderer/mdx_format.h:349-413`
// ---------------------------------------------------------------------------

/// `mdxaHeader_t` field order: `ident,version:i32` (8) + `name[MAX_QPATH]` +
/// `fScale:f32` (4) then `numFrames`.
const MDXA_NUM_FRAMES_OFFSET: usize = 4 + 4 + MAX_QPATH + 4;
/// `numFrames` (4) then `ofsFrames`.
const MDXA_OFS_FRAMES_OFFSET: usize = MDXA_NUM_FRAMES_OFFSET + 4;
/// `ofsFrames` (4) then `numBones`.
const MDXA_NUM_BONES_OFFSET: usize = MDXA_OFS_FRAMES_OFFSET + 4;
/// `numBones` (4) then `ofsCompBonePool`.
const MDXA_OFS_COMP_BONE_POOL_OFFSET: usize = MDXA_NUM_BONES_OFFSET + 4;
/// `ofsCompBonePool` (4) + `ofsSkel` (4) + `ofsEnd` (4) = `sizeof(mdxaHeader_t)`,
/// matching Raven's own `(byte*)mdxa + sizeof(mdxaHeader_t)` arithmetic
/// (`:1687`).
const MDXA_HEADER_SIZE: usize = MDXA_OFS_COMP_BONE_POOL_OFFSET + 4 + 4 + 4;
/// `mdxaSkel_t::BasePoseMat` offset: `name[MAX_QPATH]`(64) + `flags`(4) +
/// `parent`(4) precede it.
const SKEL_OFS_BASE_POSE_MAT: usize = MAX_QPATH + 4 + 4;
/// `mdxaSkel_t::BasePoseMatInv` offset: `BasePoseMat` (48 bytes, `mdxaBone_t`)
/// precedes it.
const SKEL_OFS_BASE_POSE_MAT_INV: usize = SKEL_OFS_BASE_POSE_MAT + 48;
/// `mdxaCompQuatBone_t::Comp` size (`mdx_format.h:120`) — 14 raw bytes, no
/// padding (a `char[14]` member alone forces 1-byte struct alignment).
const MDXA_COMP_QUAT_BONE_SIZE: usize = 14;

/// Raven `mdxaHeader_t->numFrames` (`:1615` and throughout the clamp checks).
///
/// # Safety
/// `header` must be a valid, non-null `EngineHost::model_mdxa` block pointer.
unsafe fn mdxa_num_frames(header: *const c_void) -> i32 {
    core::ptr::read_unaligned((header as *const u8).add(MDXA_NUM_FRAMES_OFFSET) as *const i32)
}

/// Raven `(mdxaSkelOffsets_t*)((byte*)mdxa + sizeof(mdxaHeader_t))->offsets[i]`
/// then `(mdxaSkel_t*)((byte*)mdxa + sizeof(mdxaHeader_t) + offset)` —
/// `tr_ghoul2.cpp:1815-1817`.
///
/// # Safety
/// `header` must be a valid, non-null `EngineHost::model_mdxa` block pointer
/// and `bone_index` must be `< numBones`.
unsafe fn mdxa_skel_ptr(header: *const c_void, bone_index: i32) -> *const u8 {
    let base = (header as *const u8).add(MDXA_HEADER_SIZE);
    let skel_offset = core::ptr::read_unaligned((base as *const i32).add(bone_index as usize));
    base.offset(skel_offset as isize)
}

/// Raven `skel->BasePoseMat` — `tr_ghoul2.cpp:1837` etc.
///
/// # Safety
/// Same preconditions as [`mdxa_skel_ptr`].
unsafe fn mdxa_skel_base_pose_mat(header: *const c_void, bone_index: i32) -> mdxaBone_t {
    let skel = mdxa_skel_ptr(header, bone_index);
    core::ptr::read_unaligned(skel.add(SKEL_OFS_BASE_POSE_MAT) as *const mdxaBone_t)
}

/// Raven `skel->BasePoseMatInv` — `tr_ghoul2.cpp:1855` etc.
///
/// # Safety
/// Same preconditions as [`mdxa_skel_ptr`].
unsafe fn mdxa_skel_base_pose_mat_inv(header: *const c_void, bone_index: i32) -> mdxaBone_t {
    let skel = mdxa_skel_ptr(header, bone_index);
    core::ptr::read_unaligned(skel.add(SKEL_OFS_BASE_POSE_MAT_INV) as *const mdxaBone_t)
}

/// Raven `VectorLength((float*)&temp)` applied to a matrix's first row
/// (`tr_ghoul2.cpp:1867,1955,1985` etc. — the non-`_XBOX` arm,
/// `oracle/codemp/game/q_shared.h:1487`): `sqrt(v[0]^2+v[1]^2+v[2]^2)` over
/// the first three floats of the `mdxaBone_t*` cast, i.e. `matrix[0][0..2]`.
fn vector_length3(x: f32, y: f32, z: f32) -> f32 {
    (x * x + y * y + z * z).sqrt()
}

/// Raven `void G2_TransformBone(int child,CBoneCache &BC)` — the core
/// per-bone evaluator `CBoneCache::EvalLow` drives: resolves bone-list
/// override flags (angle override, anim blend, anim override via
/// `G2_TimingModel`), clamps `newFrame`/`currentFrame`/`blendFrame`/
/// `blendOldFrame` into range, then LERPs the frame data into a quaternion
/// (`G2_CreateQuaterion`) and back (`G2_CreateMatrixFromQuaterion`), chaining
/// through `Multiply_3x4Matrix` against the parent bone's matrix to produce
/// `BC.mFinalBones[child].bone_matrix`.
///
/// `HackadelicOnClient` is const-`false` server-side (`## Raven ground
/// truth`/divergences list): every `if (HackadelicOnClient) {...} else
/// {...}` arm below folds permanently to its `else` (C10) — the `boneOverride
/// .newMatrix`/client-side reads are dead and not transcribed.
///
/// Debug-only `assert(...)` bounds checks (NDEBUG in the WinDed build) are
/// not transcribed as runtime effects, matching this crate's existing
/// treatment of Raven `assert(...)` throughout. The `DEBUG_G2_TIMING`
/// print-timing block (`0` in the Raven ground-truth build) and the
/// commented-out `r_Ghoul2UnSqash`/blend-lerp-off tail are dead code, not
/// transcribed.
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:1541-2051`
pub fn g2_transform_bone(bc: &mut CBoneCache, child: i32) {
    let child_idx = child as usize;
    let mut angle_override: i32 = 0;

    // SAFETY: `root_bone_list` is set fresh each `G2_TransformGhoulBones`
    // call into the owning `CGhoul2Info::blist` (`render/bone_cache.rs`) and
    // stays valid for the whole `EvalLow` recursion that calls this
    // function; every deref below shares that same precondition.
    let bone_list_index = crate::bones::g2_find_bone_in_list(unsafe { &*bc.root_bone_list }, child);

    // `angle_override` is only ever assigned inside this `if`, so every
    // later `angle_override & X != 0` check implies `bone_list_index >= 0` —
    // matching Raven's unchecked re-use of `boneListIndex` in those branches.
    if bone_list_index != -1 {
        let bone_list_idx = bone_list_index as usize;
        let bone_override: &mut boneInfo_t = unsafe { &mut (*bc.root_bone_list)[bone_list_idx] };

        // do we override the rotational angles?
        if bone_override.flags & BONE_ANGLES_TOTAL != 0 {
            angle_override = bone_override.flags & BONE_ANGLES_TOTAL;
        }

        // set blending stuff if we need to
        if bone_override.flags & BONE_ANIM_BLEND != 0 {
            let blend_time = bc.incoming_time as f32 - bone_override.blendStart as f32;
            let tb = &mut bc.bones[child_idx];
            // only set up the blend anim if we actually have some blend time
            // left on this bone anim - otherwise we might corrupt some blend
            // higher up the hierarchy
            if blend_time >= 0.0 && blend_time < bone_override.blendTime as f32 {
                tb.blend_frame = bone_override.blendFrame;
                tb.blend_old_frame = bone_override.blendLerpFrame;
                tb.blend_lerp = blend_time / bone_override.blendTime as f32;
                tb.blend_mode = true;
            } else {
                tb.blend_mode = false;
            }
        } else if bone_override.flags & (BONE_ANIM_OVERRIDE_LOOP | BONE_ANIM_OVERRIDE) != 0 {
            // turn off blending if we are just doing a straight animation override
            bc.bones[child_idx].blend_mode = false;
        }

        // should this animation be overridden by an animation in the bone list?
        if bone_override.flags & (BONE_ANIM_OVERRIDE_LOOP | BONE_ANIM_OVERRIDE) != 0 {
            // SAFETY: same as the function-level note above.
            let num_frames = unsafe { mdxa_num_frames(bc.header) };
            let tb = &mut bc.bones[child_idx];
            let (mut current_frame, mut new_frame, mut backlerp) =
                (tb.current_frame, tb.new_frame, tb.backlerp);
            g2_timing_model(
                bone_override,
                bc.incoming_time,
                num_frames,
                &mut current_frame,
                &mut new_frame,
                &mut backlerp,
            );
            let tb = &mut bc.bones[child_idx];
            tb.current_frame = current_frame;
            tb.new_frame = new_frame;
            tb.backlerp = backlerp;
        }
    }

    // figure out where the location of the bone animation data is
    // SAFETY: same as the function-level note above.
    let num_frames = unsafe { mdxa_num_frames(bc.header) };
    {
        let tb = &mut bc.bones[child_idx];
        if !(tb.new_frame >= 0 && tb.new_frame < num_frames) {
            tb.new_frame = 0;
        }
        if !(tb.current_frame >= 0 && tb.current_frame < num_frames) {
            tb.current_frame = 0;
        }
        // figure out where the location of the blended animation data is
        if tb.blend_frame < 0.0 || tb.blend_frame >= (num_frames + 1) as f32 {
            tb.blend_frame = 0.0;
        }
        if !(tb.blend_old_frame >= 0 && tb.blend_old_frame < num_frames) {
            tb.blend_old_frame = 0;
        }
    }

    let (blend_mode, blend_frame, blend_old_frame, blend_lerp, backlerp, current_frame, new_frame) = {
        let tb = &bc.bones[child_idx];
        (
            tb.blend_mode,
            tb.blend_frame,
            tb.blend_old_frame,
            tb.blend_lerp,
            tb.backlerp,
            tb.current_frame,
            tb.new_frame,
        )
    };

    // decide where the transformed bone is going. `tbone[6]` is Raven's
    // `static mdxaBone_t tbone[6]` scratch array (`:1544`) — every element
    // used below is written before being read within this same call, so a
    // stack-local array reproduces identical behavior without a hidden
    // static (porting-rules §B3).
    let mut tbone = [mdxaBone_t {
        matrix: [[0.0; 4]; 3],
    }; 6];

    // are we blending with another frame of anim?
    if blend_mode {
        let back = blend_frame - (blend_frame as i32) as f32;
        let front = 1.0 - back;

        uncompress_bone(&mut tbone[3].matrix, child, bc.header, blend_frame as i32);
        uncompress_bone(&mut tbone[4].matrix, child, bc.header, blend_old_frame);

        for r in 0..3 {
            for c in 0..4 {
                tbone[5].matrix[r][c] =
                    back * tbone[3].matrix[r][c] + front * tbone[4].matrix[r][c];
            }
        }
    }

    // lerp this bone - use the temp space on the ref entity to put the bone
    // transforms into
    if backlerp == 0.0 {
        uncompress_bone(&mut tbone[2].matrix, child, bc.header, current_frame);

        // blend in the other frame if we need to
        if blend_mode {
            let blend_frontlerp = 1.0 - blend_lerp;
            for r in 0..3 {
                for c in 0..4 {
                    tbone[2].matrix[r][c] = blend_lerp * tbone[2].matrix[r][c]
                        + blend_frontlerp * tbone[5].matrix[r][c];
                }
            }
        }

        if child == 0 {
            // now multiply by the root matrix, so we can offset this model
            // should we need to
            let root_matrix = bc.root_matrix;
            multiply_3x4_matrix(
                &mut bc.final_bones[child_idx].bone_matrix,
                &root_matrix,
                &tbone[2],
            );
        }
    } else {
        let frontlerp = 1.0 - backlerp;
        uncompress_bone(&mut tbone[0].matrix, child, bc.header, new_frame);
        uncompress_bone(&mut tbone[1].matrix, child, bc.header, current_frame);

        for r in 0..3 {
            for c in 0..4 {
                tbone[2].matrix[r][c] =
                    backlerp * tbone[0].matrix[r][c] + frontlerp * tbone[1].matrix[r][c];
            }
        }

        // blend in the other frame if we need to
        if blend_mode {
            let blend_frontlerp = 1.0 - blend_lerp;
            for r in 0..3 {
                for c in 0..4 {
                    tbone[2].matrix[r][c] = blend_lerp * tbone[2].matrix[r][c]
                        + blend_frontlerp * tbone[5].matrix[r][c];
                }
            }
        }

        if child == 0 {
            // now multiply by the root matrix, so we can offset this model
            // should we need to
            let root_matrix = bc.root_matrix;
            multiply_3x4_matrix(
                &mut bc.final_bones[child_idx].bone_matrix,
                &root_matrix,
                &tbone[2],
            );
        }
    }

    let parent = bc.final_bones[child_idx].parent;

    if angle_override & BONE_ANGLES_REPLACE != 0 {
        let is_rag =
            (angle_override & BONE_ANGLES_RAGDOLL) != 0 || (angle_override & BONE_ANGLES_IK) != 0;

        // SAFETY: same as the function-level note above; `bone_list_index`
        // is non-negative here (see the invariant note at the top).
        let base_pose_mat = unsafe { mdxa_skel_base_pose_mat(bc.header, child) };
        let base_pose_mat_inv = unsafe { mdxa_skel_base_pose_mat_inv(bc.header, child) };
        let bone_override_matrix = unsafe { (*bc.root_bone_list)[bone_list_index as usize].matrix };

        if is_rag {
            let parent_matrix = bc.final_bones[parent as usize].bone_matrix;
            // give us the matrix the animation thinks we should have, so we
            // can get the correct X&Y coords
            let mut first_pass = mdxaBone_t {
                matrix: [[0.0; 4]; 3],
            };
            multiply_3x4_matrix(&mut first_pass, &parent_matrix, &tbone[2]);

            // this is crazy, we are gonna drive the animation to ID while we
            // are doing post mults to compensate.
            let mut temp = mdxaBone_t {
                matrix: [[0.0; 4]; 3],
            };
            multiply_3x4_matrix(&mut temp, &first_pass, &base_pose_mat);
            let matrix_scale =
                vector_length3(temp.matrix[0][0], temp.matrix[0][1], temp.matrix[0][2]);

            let mut to_matrix = mdxaBone_t {
                matrix: [[0.0; 4]; 3],
            };
            to_matrix.matrix[0][0] = matrix_scale;
            to_matrix.matrix[1][1] = matrix_scale;
            to_matrix.matrix[2][2] = matrix_scale;
            to_matrix.matrix[0][3] = temp.matrix[0][3];
            to_matrix.matrix[1][3] = temp.matrix[1][3];
            to_matrix.matrix[2][3] = temp.matrix[2][3];

            multiply_3x4_matrix(&mut temp, &to_matrix, &base_pose_mat_inv); // dest first arg

            // SAFETY: same as above.
            let (bone_blend_start, bone_blend_time) = unsafe {
                let bone = &(*bc.root_bone_list)[bone_list_index as usize];
                (bone.boneBlendStart, bone.boneBlendTime)
            };
            let blend_time = bc.incoming_time as f32 - bone_blend_start as f32;
            let this_blend_lerp = blend_time / bone_blend_time as f32;

            if this_blend_lerp > 0.0 {
                // has started
                if this_blend_lerp > 1.0 {
                    // done
                    bc.final_bones[child_idx].bone_matrix = temp;
                } else {
                    // now do the blend into the destination
                    let blend_frontlerp = 1.0 - this_blend_lerp;
                    let mut result = mdxaBone_t {
                        matrix: [[0.0; 4]; 3],
                    };
                    for r in 0..3 {
                        for c in 0..4 {
                            result.matrix[r][c] = this_blend_lerp * temp.matrix[r][c]
                                + blend_frontlerp * tbone[2].matrix[r][c];
                        }
                    }
                    bc.final_bones[child_idx].bone_matrix = result;
                }
            }
            // `this_blend_lerp <= 0.0`: Raven leaves `BC.mFinalBones[child]
            // .boneMatrix` unwritten in this arm — not transcribed as a write.
        } else {
            let parent_matrix = bc.final_bones[parent as usize].bone_matrix;
            // give us the matrix the animation thinks we should have, so we
            // can get the correct X&Y coords
            let mut first_pass = mdxaBone_t {
                matrix: [[0.0; 4]; 3],
            };
            multiply_3x4_matrix(&mut first_pass, &parent_matrix, &tbone[2]);

            // SAFETY: same as above.
            let (bone_blend_time, bone_blend_start) = unsafe {
                let bone = &(*bc.root_bone_list)[bone_list_index as usize];
                (bone.boneBlendTime, bone.boneBlendStart)
            };

            // are we attempting to blend with the base animation? and still
            // within blend time?
            if bone_blend_time != 0 && (bone_blend_time + bone_blend_start) < bc.incoming_time {
                // ok, we are supposed to be blending. Work out lerp
                let blend_time = bc.incoming_time as f32 - bone_blend_start as f32;
                let this_blend_lerp = blend_time / bone_blend_time as f32;

                if this_blend_lerp <= 1.0 {
                    // now work out the matrix we want to get *to* - firstPass
                    // is where we are coming *from*
                    let mut temp = mdxaBone_t {
                        matrix: [[0.0; 4]; 3],
                    };
                    multiply_3x4_matrix(&mut temp, &first_pass, &base_pose_mat);
                    let matrix_scale =
                        vector_length3(temp.matrix[0][0], temp.matrix[0][1], temp.matrix[0][2]);

                    let mut new_matrix_temp = mdxaBone_t {
                        matrix: [[0.0; 4]; 3],
                    };
                    for i in 0..3 {
                        for x in 0..3 {
                            new_matrix_temp.matrix[i][x] =
                                bone_override_matrix.matrix[i][x] * matrix_scale;
                        }
                    }
                    new_matrix_temp.matrix[0][3] = temp.matrix[0][3];
                    new_matrix_temp.matrix[1][3] = temp.matrix[1][3];
                    new_matrix_temp.matrix[2][3] = temp.matrix[2][3];

                    multiply_3x4_matrix(&mut temp, &new_matrix_temp, &base_pose_mat_inv);

                    // now do the blend into the destination
                    let blend_frontlerp = 1.0 - this_blend_lerp;
                    let mut result = mdxaBone_t {
                        matrix: [[0.0; 4]; 3],
                    };
                    for r in 0..3 {
                        for c in 0..4 {
                            result.matrix[r][c] = this_blend_lerp * temp.matrix[r][c]
                                + blend_frontlerp * first_pass.matrix[r][c];
                        }
                    }
                    bc.final_bones[child_idx].bone_matrix = result;
                } else {
                    bc.final_bones[child_idx].bone_matrix = first_pass;
                }
            } else {
                // no, so just override it directly
                let mut temp = mdxaBone_t {
                    matrix: [[0.0; 4]; 3],
                };
                multiply_3x4_matrix(&mut temp, &first_pass, &base_pose_mat);
                let matrix_scale =
                    vector_length3(temp.matrix[0][0], temp.matrix[0][1], temp.matrix[0][2]);

                let mut new_matrix_temp = mdxaBone_t {
                    matrix: [[0.0; 4]; 3],
                };
                for i in 0..3 {
                    for x in 0..3 {
                        new_matrix_temp.matrix[i][x] =
                            bone_override_matrix.matrix[i][x] * matrix_scale;
                    }
                }
                new_matrix_temp.matrix[0][3] = temp.matrix[0][3];
                new_matrix_temp.matrix[1][3] = temp.matrix[1][3];
                new_matrix_temp.matrix[2][3] = temp.matrix[2][3];

                multiply_3x4_matrix(
                    &mut bc.final_bones[child_idx].bone_matrix,
                    &new_matrix_temp,
                    &base_pose_mat_inv,
                );
            }
        }
    } else if angle_override & BONE_ANGLES_PREMULT != 0 {
        // SAFETY: same invariant as above.
        let bone_override_matrix = unsafe { (*bc.root_bone_list)[bone_list_index as usize].matrix };

        if (angle_override & BONE_ANGLES_RAGDOLL) != 0 || (angle_override & BONE_ANGLES_IK) != 0 {
            let mut tmp = mdxaBone_t {
                matrix: [[0.0; 4]; 3],
            };
            if child == 0 {
                let root_matrix = bc.root_matrix;
                multiply_3x4_matrix(&mut tmp, &root_matrix, &bone_override_matrix);
            } else {
                let parent_matrix = bc.final_bones[parent as usize].bone_matrix;
                multiply_3x4_matrix(&mut tmp, &parent_matrix, &bone_override_matrix);
            }
            multiply_3x4_matrix(&mut bc.final_bones[child_idx].bone_matrix, &tmp, &tbone[2]);
        } else if child == 0 {
            // use the incoming root matrix as our basis
            let root_matrix = bc.root_matrix;
            multiply_3x4_matrix(
                &mut bc.final_bones[child_idx].bone_matrix,
                &root_matrix,
                &bone_override_matrix,
            );
        } else {
            // convert from 3x4 matrix to a 4x4 matrix
            let parent_matrix = bc.final_bones[parent as usize].bone_matrix;
            multiply_3x4_matrix(
                &mut bc.final_bones[child_idx].bone_matrix,
                &parent_matrix,
                &bone_override_matrix,
            );
        }
    } else if child != 0 {
        // now transform the matrix by its parent, assuming we have a parent,
        // and we aren't overriding the angles absolutely
        let parent_matrix = bc.final_bones[parent as usize].bone_matrix;
        multiply_3x4_matrix(
            &mut bc.final_bones[child_idx].bone_matrix,
            &parent_matrix,
            &tbone[2],
        );
    }

    // now multiply our resulting bone by an override matrix should we need to
    if angle_override & BONE_ANGLES_POSTMULT != 0 {
        // SAFETY: same invariant as above.
        let bone_override_matrix = unsafe { (*bc.root_bone_list)[bone_list_index as usize].matrix };
        let temp_matrix = bc.final_bones[child_idx].bone_matrix;
        multiply_3x4_matrix(
            &mut bc.final_bones[child_idx].bone_matrix,
            &temp_matrix,
            &bone_override_matrix,
        );
    }
}

/// Raven `void G2_TimingModel(boneInfo_t &bone,int currentTime,int
/// numFramesInFile,int &currentFrame,int &newFrame,float &lerp)` — third
/// file-local faithful duplicate; see the module doc's first finding. Unlike
/// `bones.rs`'s copy, this one takes `&mut boneInfo_t` (this file's call site
/// supplies one) and so transcribes Raven's one write
/// (`bone.flags &= ~(BONE_ANIM_TOTAL)`, the "not override-loop, not
/// override-freeze, ran off the end" arm) faithfully.
///
/// Debug-only `assert` bounds checks are not transcribed as runtime effects.
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:1167-1407`
fn g2_timing_model(
    bone: &mut boneInfo_t,
    current_time: i32,
    num_frames_in_file: i32,
    current_frame: &mut i32,
    new_frame: &mut i32,
    lerp: &mut f32,
) {
    let anim_speed = bone.animSpeed;
    let mut time = if bone.pauseTime != 0 {
        (bone.pauseTime - bone.startTime) as f32 / 50.0
    } else {
        (current_time - bone.startTime) as f32 / 50.0
    };
    if time < 0.0 {
        time = 0.0;
    }
    let mut new_frame_g = bone.startFrame as f32 + (time * anim_speed);

    let anim_size = bone.endFrame - bone.startFrame;
    // Raven shadows `endFrame` with this float twin (`(float)bone.endFrame`)
    // for the rest of the function; every bare `endFrame` reference below is
    // this local, not `bone.endFrame` directly.
    let end_frame = bone.endFrame as f32;

    if anim_size != 0 {
        if (anim_speed > 0.0 && new_frame_g > end_frame - 1.0)
            || (anim_speed < 0.0 && new_frame_g < end_frame + 1.0)
        {
            if bone.flags & BONE_ANIM_OVERRIDE_LOOP != 0 {
                if anim_speed < 0.0 {
                    if new_frame_g < end_frame + 1.0 && new_frame_g >= end_frame {
                        *lerp = (end_frame + 1.0) - new_frame_g;
                        *current_frame = end_frame as i32;
                        *new_frame = bone.startFrame;
                    } else {
                        if new_frame_g <= end_frame + 1.0 {
                            new_frame_g = end_frame
                                + (new_frame_g - end_frame) % (anim_size as f32)
                                - anim_size as f32;
                        }
                        *lerp = new_frame_g.ceil() - new_frame_g;
                        *current_frame = new_frame_g.ceil() as i32;
                        if *current_frame as f32 <= end_frame + 1.0 {
                            *new_frame = bone.startFrame;
                        } else {
                            *new_frame = *current_frame - 1;
                        }
                    }
                } else if new_frame_g > end_frame - 1.0 && new_frame_g < end_frame {
                    *lerp = new_frame_g - (new_frame_g as i32) as f32;
                    *current_frame = new_frame_g as i32;
                    *new_frame = bone.startFrame;
                } else {
                    if new_frame_g >= end_frame {
                        new_frame_g = end_frame + (new_frame_g - end_frame) % (anim_size as f32)
                            - anim_size as f32;
                    }
                    *lerp = new_frame_g - (new_frame_g as i32) as f32;
                    *current_frame = new_frame_g as i32;
                    if new_frame_g >= end_frame - 1.0 {
                        *new_frame = bone.startFrame;
                    } else {
                        *new_frame = *current_frame + 1;
                    }
                }
            } else if bone.flags & BONE_ANIM_OVERRIDE_FREEZE == BONE_ANIM_OVERRIDE_FREEZE {
                if anim_speed > 0.0 {
                    *current_frame = bone.endFrame - 1;
                } else {
                    *current_frame = bone.endFrame + 1;
                }
                *new_frame = *current_frame;
                *lerp = 0.0;
            } else {
                // if we are supposed to reset the default anim, then do so
                bone.flags &= !BONE_ANIM_TOTAL;
            }
        } else if anim_speed > 0.0 {
            *current_frame = new_frame_g as i32;
            *lerp = new_frame_g - *current_frame as f32;
            *new_frame = *current_frame + 1;
            if *new_frame >= end_frame as i32 {
                if bone.flags & BONE_ANIM_OVERRIDE_LOOP != 0 {
                    *new_frame = bone.startFrame;
                } else {
                    *new_frame = bone.endFrame - 1;
                }
            }
        } else {
            *lerp = new_frame_g.ceil() - new_frame_g;
            *current_frame = new_frame_g.ceil() as i32;
            if *current_frame > bone.startFrame {
                *current_frame = bone.startFrame;
                *new_frame = *current_frame;
                *lerp = 0.0;
            } else {
                *new_frame = *current_frame - 1;
                if (*new_frame as f32) < end_frame + 1.0 {
                    if bone.flags & BONE_ANIM_OVERRIDE_LOOP != 0 {
                        *new_frame = bone.startFrame;
                    } else {
                        *new_frame = bone.endFrame + 1;
                    }
                }
            }
        }
    } else {
        if anim_speed < 0.0 {
            *current_frame = bone.endFrame + 1;
        } else {
            *current_frame = bone.endFrame - 1;
        }
        if *current_frame < 0 {
            *current_frame = 0;
        }
        *new_frame = *current_frame;
        *lerp = 0.0;
    }
    let _ = num_frames_in_file; // only consulted by the debug-only asserts above
}

/// Raven `void Multiply_3x4Matrix(mdxaBone_t *out, mdxaBone_t *in2, mdxaBone_t
/// *in)` — the 3x4 (rotation + translation) matrix multiply `G2_TransformBone`
/// chains bone-to-parent transforms through. Compiled `-ffp-contract=off`
/// (`G2SV-D6`) so the multiply-add sequence must not fuse into FMA — transcribe
/// the exact per-element expression order, not an equivalent refactor.
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:1128-1145`
pub fn multiply_3x4_matrix(out: &mut mdxaBone_t, in2: &mdxaBone_t, inm: &mdxaBone_t) {
    // first row of out
    out.matrix[0][0] = (in2.matrix[0][0] * inm.matrix[0][0])
        + (in2.matrix[0][1] * inm.matrix[1][0])
        + (in2.matrix[0][2] * inm.matrix[2][0]);
    out.matrix[0][1] = (in2.matrix[0][0] * inm.matrix[0][1])
        + (in2.matrix[0][1] * inm.matrix[1][1])
        + (in2.matrix[0][2] * inm.matrix[2][1]);
    out.matrix[0][2] = (in2.matrix[0][0] * inm.matrix[0][2])
        + (in2.matrix[0][1] * inm.matrix[1][2])
        + (in2.matrix[0][2] * inm.matrix[2][2]);
    out.matrix[0][3] = (in2.matrix[0][0] * inm.matrix[0][3])
        + (in2.matrix[0][1] * inm.matrix[1][3])
        + (in2.matrix[0][2] * inm.matrix[2][3])
        + in2.matrix[0][3];
    // second row of out
    out.matrix[1][0] = (in2.matrix[1][0] * inm.matrix[0][0])
        + (in2.matrix[1][1] * inm.matrix[1][0])
        + (in2.matrix[1][2] * inm.matrix[2][0]);
    out.matrix[1][1] = (in2.matrix[1][0] * inm.matrix[0][1])
        + (in2.matrix[1][1] * inm.matrix[1][1])
        + (in2.matrix[1][2] * inm.matrix[2][1]);
    out.matrix[1][2] = (in2.matrix[1][0] * inm.matrix[0][2])
        + (in2.matrix[1][1] * inm.matrix[1][2])
        + (in2.matrix[1][2] * inm.matrix[2][2]);
    out.matrix[1][3] = (in2.matrix[1][0] * inm.matrix[0][3])
        + (in2.matrix[1][1] * inm.matrix[1][3])
        + (in2.matrix[1][2] * inm.matrix[2][3])
        + in2.matrix[1][3];
    // third row of out
    out.matrix[2][0] = (in2.matrix[2][0] * inm.matrix[0][0])
        + (in2.matrix[2][1] * inm.matrix[1][0])
        + (in2.matrix[2][2] * inm.matrix[2][0]);
    out.matrix[2][1] = (in2.matrix[2][0] * inm.matrix[0][1])
        + (in2.matrix[2][1] * inm.matrix[1][1])
        + (in2.matrix[2][2] * inm.matrix[2][1]);
    out.matrix[2][2] = (in2.matrix[2][0] * inm.matrix[0][2])
        + (in2.matrix[2][1] * inm.matrix[1][2])
        + (in2.matrix[2][2] * inm.matrix[2][2]);
    out.matrix[2][3] = (in2.matrix[2][0] * inm.matrix[0][3])
        + (in2.matrix[2][1] * inm.matrix[1][3])
        + (in2.matrix[2][2] * inm.matrix[2][3])
        + in2.matrix[2][3];
}

/// Raven `void G2_CreateQuaterion(mdxaBone_t *mat, vec4_t quat)` — derives a
/// quaternion from a 3x3 rotation sub-matrix (Shepperd's method: trace-based
/// fast path when `t > 0.00000001`, else the largest-diagonal-element case
/// split). `mat` is read-only; `quat` is the out-param.
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:1048-1095`
pub fn g2_create_quaterion(mat: &mdxaBone_t, quat: &mut vec4_t) {
    // this is revised for the 3x4 matrix we use in G2.
    let t = 1.0 + mat.matrix[0][0] + mat.matrix[1][1] + mat.matrix[2][2];

    // If the trace of the matrix is greater than zero, then perform an
    // "instant" calculation.
    // Important note wrt. rounding errors: Test if (T > 0.00000001) to avoid
    // large distortions!
    if t > 0.00000001 {
        let s = t.sqrt() * 2.0;
        quat[0] = (mat.matrix[1][2] - mat.matrix[2][1]) / s;
        quat[1] = (mat.matrix[2][0] - mat.matrix[0][2]) / s;
        quat[2] = (mat.matrix[0][1] - mat.matrix[1][0]) / s;
        quat[3] = 0.25 * s;
    } else {
        // If the trace of the matrix is equal to zero then identify which
        // major diagonal element has the greatest value. Depending on this,
        // calculate the following:
        if mat.matrix[0][0] > mat.matrix[1][1] && mat.matrix[0][0] > mat.matrix[2][2] {
            // Column 0:
            let s = (1.0 + mat.matrix[0][0] - mat.matrix[1][1] - mat.matrix[2][2]).sqrt() * 2.0;
            quat[0] = 0.25 * s;
            quat[1] = (mat.matrix[0][1] + mat.matrix[1][0]) / s;
            quat[2] = (mat.matrix[2][0] + mat.matrix[0][2]) / s;
            quat[3] = (mat.matrix[1][2] - mat.matrix[2][1]) / s;
        } else if mat.matrix[1][1] > mat.matrix[2][2] {
            // Column 1:
            let s = (1.0 + mat.matrix[1][1] - mat.matrix[0][0] - mat.matrix[2][2]).sqrt() * 2.0;
            quat[0] = (mat.matrix[0][1] + mat.matrix[1][0]) / s;
            quat[1] = 0.25 * s;
            quat[2] = (mat.matrix[1][2] + mat.matrix[2][1]) / s;
            quat[3] = (mat.matrix[2][0] - mat.matrix[0][2]) / s;
        } else {
            // Column 2:
            let s = (1.0 + mat.matrix[2][2] - mat.matrix[0][0] - mat.matrix[1][1]).sqrt() * 2.0;
            quat[0] = (mat.matrix[2][0] + mat.matrix[0][2]) / s;
            quat[1] = (mat.matrix[1][2] + mat.matrix[2][1]) / s;
            quat[2] = 0.25 * s;
            quat[3] = (mat.matrix[0][1] - mat.matrix[1][0]) / s;
        }
    }
}

/// Raven `void G2_CreateMatrixFromQuaterion(mdxaBone_t *mat, vec4_t quat)` —
/// the inverse of `G2_CreateQuaterion`: rebuilds the 3x3 rotation sub-matrix
/// from a quaternion, zeroing the translation column. `quat` is read-only;
/// `mat` is the out-param.
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:1097-1125`
pub fn g2_create_matrix_from_quaterion(mat: &mut mdxaBone_t, quat: &vec4_t) {
    let xx = quat[0] * quat[0];
    let xy = quat[0] * quat[1];
    let xz = quat[0] * quat[2];
    let xw = quat[0] * quat[3];

    let yy = quat[1] * quat[1];
    let yz = quat[1] * quat[2];
    let yw = quat[1] * quat[3];

    let zz = quat[2] * quat[2];
    let zw = quat[2] * quat[3];

    mat.matrix[0][0] = 1.0 - 2.0 * (yy + zz);
    mat.matrix[1][0] = 2.0 * (xy - zw);
    mat.matrix[2][0] = 2.0 * (xz + yw);

    mat.matrix[0][1] = 2.0 * (xy + zw);
    mat.matrix[1][1] = 1.0 - 2.0 * (xx + zz);
    mat.matrix[2][1] = 2.0 * (yz - xw);

    mat.matrix[0][2] = 2.0 * (xz - yw);
    mat.matrix[1][2] = 2.0 * (yz + xw);
    mat.matrix[2][2] = 1.0 - 2.0 * (xx + yy);

    mat.matrix[0][3] = 0.0;
    mat.matrix[1][3] = 0.0;
    mat.matrix[2][3] = 0.0;
}

/// Raven `static int G2_GetBonePoolIndex(const mdxaHeader_t *pMDXAHeader, int
/// iFrame, int iBone)` (private helper of `UnCompressBone`) — computes the
/// compressed-bone-pool slot for `(frame, bone)`: `iOffsetToIndex = (iFrame *
/// numBones * 3) + (iBone * 3)` bytes into the header's `ofsFrames` block,
/// read as an `mdxaIndex_t`, masked to `iIndex & 0x00FFFFFF` (the top byte is
/// non-index payload; masking off it is noted upstream as an unfixed
/// big-endian hazard, kept faithfully). `header` is the same opaque
/// `*mut c_void` model memory as `CBoneCache::header` (`G2SV-D5`: this crate
/// never names `mdxaHeader_t`/`mdxaIndex_t`).
///
/// Not enumerated in this file's roster/method-table row (doc/oracle
/// mismatch — reported, not improvised around): see the module-doc finding.
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:1148-1155`
fn g2_get_bone_pool_index(header: *const c_void, frame: i32, bone: i32) -> i32 {
    // Safety: `header` is a valid, non-null `EngineHost::model_mdxa` block
    // pointer for the whole `CBoneCache` lifetime (`G2SV-D5`, ruling 36).
    unsafe {
        let num_bones = core::ptr::read_unaligned(
            (header as *const u8).add(MDXA_NUM_BONES_OFFSET) as *const i32
        );
        let ofs_frames = core::ptr::read_unaligned(
            (header as *const u8).add(MDXA_OFS_FRAMES_OFFSET) as *const i32,
        );
        let offset_to_index = (frame * num_bones * 3) + (bone * 3);
        let index_ptr = (header as *const u8)
            .offset(ofs_frames as isize)
            .offset(offset_to_index as isize) as *const i32;
        let i_index = core::ptr::read_unaligned(index_ptr);
        i_index & 0x00FF_FFFF // this will cause problems for big-endian machines... ;-)
    }
}

/// Raven `/*static inline*/ void UnCompressBone(float mat[3][4], int
/// iBoneIndex, const mdxaHeader_t *pMDXAHeader, int iFrame)` — the thin
/// wrapper `G2_TransformBone` calls (five times, `:1707,1708,1723,1747,1748`)
/// to decompress one bone's matrix for a given frame: locates the
/// `mdxaCompQuatBone_t` pool via the header's `ofsCompBonePool` offset,
/// indexes it with `G2_GetBonePoolIndex`, and hands the 14-byte compressed
/// record to `MC_UnCompressQuat` (`matcomp.rs::mc_uncompress_quat`, landed).
/// Also the sole call site of `G2_RagGetAnimMatrix` (`:1464`, the
/// not-yet-landed `ragdoll.rs`'s domain) — stubbed here as the primary
/// (5-call) home per the module-doc finding.
///
/// Not enumerated in this file's roster/method-table row (doc/oracle
/// mismatch — reported, not improvised around): see the module-doc finding.
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:1158-1163`
pub fn uncompress_bone(
    mat: &mut [[f32; 4]; 3],
    bone_index: i32,
    header: *const c_void,
    frame: i32,
) {
    // Safety: `header` is a valid, non-null `EngineHost::model_mdxa` block
    // pointer for the whole `CBoneCache` lifetime (`G2SV-D5`, ruling 36).
    unsafe {
        let ofs_comp_bone_pool = core::ptr::read_unaligned(
            (header as *const u8).add(MDXA_OFS_COMP_BONE_POOL_OFFSET) as *const i32,
        );
        let pool_index = g2_get_bone_pool_index(header, frame, bone_index);
        let comp_ptr = (header as *const u8)
            .offset(ofs_comp_bone_pool as isize)
            .add(pool_index as usize * MDXA_COMP_QUAT_BONE_SIZE);
        let comp = core::slice::from_raw_parts(comp_ptr, MDXA_COMP_QUAT_BONE_SIZE);
        crate::matcomp::mc_uncompress_quat(mat, comp);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity() -> mdxaBone_t {
        let mut m = mdxaBone_t {
            matrix: [[0.0; 4]; 3],
        };
        m.matrix[0][0] = 1.0;
        m.matrix[1][1] = 1.0;
        m.matrix[2][2] = 1.0;
        m
    }

    #[test]
    fn multiply_3x4_matrix_identity_is_identity() {
        let id = identity();
        let mut out = mdxaBone_t {
            matrix: [[9.0; 4]; 3],
        };
        multiply_3x4_matrix(&mut out, &id, &id);
        assert_eq!(out, id);
    }

    #[test]
    fn multiply_3x4_matrix_applies_translation() {
        let id = identity();
        let mut translate = identity();
        translate.matrix[0][3] = 1.0;
        translate.matrix[1][3] = 2.0;
        translate.matrix[2][3] = 3.0;

        let mut out = mdxaBone_t {
            matrix: [[0.0; 4]; 3],
        };
        // out = translate * id -> translate's own translation column carries
        // through unchanged (Multiply_3x4Matrix's `+ in2.matrix[row][3]` term).
        multiply_3x4_matrix(&mut out, &translate, &id);
        assert_eq!(out, translate);
    }

    #[test]
    fn quaternion_matrix_round_trip_identity() {
        let id = identity();
        let mut quat: vec4_t = [0.0; 4];
        g2_create_quaterion(&id, &mut quat);
        // identity rotation -> zero vector part, w = 1
        assert!((quat[0]).abs() < 1e-6);
        assert!((quat[1]).abs() < 1e-6);
        assert!((quat[2]).abs() < 1e-6);
        assert!((quat[3] - 1.0).abs() < 1e-6);

        let mut mat = mdxaBone_t {
            matrix: [[9.0; 4]; 3],
        };
        g2_create_matrix_from_quaterion(&mut mat, &quat);
        for r in 0..3 {
            for c in 0..3 {
                assert!((mat.matrix[r][c] - id.matrix[r][c]).abs() < 1e-6);
            }
        }
        // translation column always zeroed by G2_CreateMatrixFromQuaterion
        assert_eq!(mat.matrix[0][3], 0.0);
        assert_eq!(mat.matrix[1][3], 0.0);
        assert_eq!(mat.matrix[2][3], 0.0);
    }

    #[test]
    fn quaternion_matrix_round_trip_90_deg_z() {
        // 90-degree rotation about Z: takes the largest-diagonal-element
        // branch of G2_CreateQuaterion (trace is 1, not > 0.00000001... in
        // fact trace = 1+0+0+1 = 2 here, so this actually exercises the fast
        // path; kept simple and self-checking either way).
        let mut mat = mdxaBone_t {
            matrix: [[0.0; 4]; 3],
        };
        mat.matrix[0][0] = 0.0;
        mat.matrix[0][1] = -1.0;
        mat.matrix[1][0] = 1.0;
        mat.matrix[1][1] = 0.0;
        mat.matrix[2][2] = 1.0;

        let mut quat: vec4_t = [0.0; 4];
        g2_create_quaterion(&mat, &mut quat);

        let mut back = mdxaBone_t {
            matrix: [[9.0; 4]; 3],
        };
        g2_create_matrix_from_quaterion(&mut back, &quat);

        for r in 0..3 {
            for c in 0..3 {
                assert!(
                    (back.matrix[r][c] - mat.matrix[r][c]).abs() < 1e-5,
                    "mismatch at [{r}][{c}]: {} vs {}",
                    back.matrix[r][c],
                    mat.matrix[r][c]
                );
            }
        }
    }

    /// Builds a synthetic in-memory `mdxaHeader_t` block: one frame, one
    /// bone, a single `mdxaIndex_t` entry, and one 14-byte compressed
    /// identity-quaternion-with-translation record — enough to exercise
    /// `g2_get_bone_pool_index`/`uncompress_bone`'s byte arithmetic without
    /// depending on any sibling module's still-`todo!()` body.
    #[test]
    fn uncompress_bone_reads_synthetic_header() {
        let ofs_frames = MDXA_HEADER_SIZE as i32;
        let ofs_comp_bone_pool = ofs_frames + 4; // one mdxaIndex_t (4 bytes) then the pool

        let mut buf = vec![0u8; MDXA_HEADER_SIZE + 4 + MDXA_COMP_QUAT_BONE_SIZE];
        buf[MDXA_NUM_FRAMES_OFFSET..MDXA_NUM_FRAMES_OFFSET + 4]
            .copy_from_slice(&1i32.to_ne_bytes());
        buf[MDXA_OFS_FRAMES_OFFSET..MDXA_OFS_FRAMES_OFFSET + 4]
            .copy_from_slice(&ofs_frames.to_ne_bytes());
        buf[MDXA_NUM_BONES_OFFSET..MDXA_NUM_BONES_OFFSET + 4].copy_from_slice(&1i32.to_ne_bytes());
        buf[MDXA_OFS_COMP_BONE_POOL_OFFSET..MDXA_OFS_COMP_BONE_POOL_OFFSET + 4]
            .copy_from_slice(&ofs_comp_bone_pool.to_ne_bytes());

        // mdxaIndex_t at ofsFrames: index 0 into the compressed-bone pool
        // (iOffsetToIndex = (0*1*3)+(0*3) = 0, so this IS the pool index).
        let index_pos = ofs_frames as usize;
        buf[index_pos..index_pos + 4].copy_from_slice(&0i32.to_ne_bytes());

        // A 14-byte compressed record decoding to an identity rotation +
        // zero translation: MC_UnCompressQuat reads w,x,y,z as
        // (u16/16383.0 - 2.0), so w=1,x=y=z=0 needs raw u16 = 3*16383 = 49149
        // for w and 2*16383=32766 for x/y/z; translation reads
        // (u16/64.0 - 512.0), so 0.0 needs raw u16 = 512*64 = 32768.
        let comp_pos = ofs_comp_bone_pool as usize;
        let comp = &mut buf[comp_pos..comp_pos + MDXA_COMP_QUAT_BONE_SIZE];
        comp[0..2].copy_from_slice(&49149u16.to_le_bytes());
        comp[2..4].copy_from_slice(&32766u16.to_le_bytes());
        comp[4..6].copy_from_slice(&32766u16.to_le_bytes());
        comp[6..8].copy_from_slice(&32766u16.to_le_bytes());
        comp[8..10].copy_from_slice(&32768u16.to_le_bytes());
        comp[10..12].copy_from_slice(&32768u16.to_le_bytes());
        comp[12..14].copy_from_slice(&32768u16.to_le_bytes());

        let header = buf.as_ptr() as *const c_void;
        let pool_index = g2_get_bone_pool_index(header, 0, 0);
        assert_eq!(pool_index, 0);

        let mut mat = [[9.0f32; 4]; 3];
        uncompress_bone(&mut mat, 0, header, 0);

        let id = identity();
        for r in 0..3 {
            for c in 0..4 {
                assert!((mat[r][c] - id.matrix[r][c]).abs() < 1e-3, "[{r}][{c}]");
            }
        }
    }

    #[test]
    fn g2_timing_model_steady_forward_playback() {
        let mut bone = make_bone(0, 10, 0, 1.0, 0);
        let mut current_frame = 0;
        let mut new_frame = 0;
        let mut lerp = 0.0;
        g2_timing_model(
            &mut bone,
            100,
            100,
            &mut current_frame,
            &mut new_frame,
            &mut lerp,
        );
        // time = 100/50 = 2.0, newFrame_g = 0 + 2.0*1.0 = 2.0
        assert_eq!(current_frame, 2);
        assert_eq!(new_frame, 3);
        assert!((lerp - 0.0).abs() < 1e-6);
    }

    #[test]
    fn g2_timing_model_clears_flags_when_not_looping_or_freezing() {
        // Runs off the end with neither BONE_ANIM_OVERRIDE_LOOP nor
        // BONE_ANIM_OVERRIDE_FREEZE set: Raven clears BONE_ANIM_TOTAL bits.
        let mut bone = make_bone(0, 5, BONE_ANIM_OVERRIDE, 1.0, 0);
        let mut current_frame = 0;
        let mut new_frame = 0;
        let mut lerp = 0.0;
        // time = 10000/50 = 200 -> way past endFrame(5)-1
        g2_timing_model(
            &mut bone,
            10_000,
            100,
            &mut current_frame,
            &mut new_frame,
            &mut lerp,
        );
        assert_eq!(bone.flags & BONE_ANIM_TOTAL, 0);
    }

    fn make_bone(
        start_frame: i32,
        end_frame: i32,
        flags: i32,
        anim_speed: f32,
        pause_time: i32,
    ) -> boneInfo_t {
        // SAFETY: boneInfo_t is `#[repr(C)]` POD; zero-init then set only the
        // fields this test exercises, matching the crate's other synthetic
        // fixture construction for this type.
        let mut bone: boneInfo_t = unsafe { core::mem::zeroed() };
        bone.startFrame = start_frame;
        bone.endFrame = end_frame;
        bone.flags = flags;
        bone.animSpeed = anim_speed;
        bone.pauseTime = pause_time;
        bone.startTime = 0;
        bone
    }
}
