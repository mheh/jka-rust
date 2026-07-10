//! Raven `CBoneCache`/`CTransformBone`/`SBoneCalc` (`tr_ghoul2.cpp:166-567`) —
//! the per-model bone-evaluation cache: three parallel per-bone arrays
//! (`bones`/`final_bones`/`smooth_bones`), lazily evaluated and memoized by an
//! integer `touch` generation. Owned by the arena's `Ghoul2System.bone_caches`
//! (`G2SV-D9`), keyed by `BoneCacheId` — not a raw `CBoneCache *` per
//! `CGhoul2Info` as in Raven (`ghoul2_shared.h:265`; the aliasing pointer is
//! replaced by the §B5 handle, `G2SV-D5`/`G2SV-D9`).
//!
//! Per the roster (`docs/subsystems/ghoul2-server.md`, `render/bone_cache.rs`
//! row): `CBoneCache::EvalLow`/`Eval`/`EvalRender`/`EvalUnsmooth`/`SmoothLow`/
//! `GetParent`/`WasRendered`, the ctor (parent-seeding from `mdxaSkel_t`,
//! header read via `EngineHost`), and the free functions
//! `EvalBoneCache`/`RemoveBoneCache`.
//!
//! `mod`/`header` are loader model memory: per `G2SV-D5` this crate never
//! names `model_t`/`mdxaHeader_t`, so `header` is the raw
//! `EngineHost::model_mdxa` block pointer and `model` is the `qhandle_t` the
//! ctor received (Raven's `const model_t *mod` collapses to the handle it was
//! built from — the live, non-`#if 0` code path never dereferences it as a
//! `model_t` itself).
//!
//! Dropped, `_XBOX`-only surface (`_XBOX` OFF, doc's Raven-ground-truth build
//! config): `SetRenderMatrix` (private helper, `:208-234`), the
//! `Z_Malloc`-backed raw-array member arm (`:360-372`; the `vector<>` arm is
//! live), `EvalFull` (`:557-566`), and the `_XBOX` destructor (`:432-439`) —
//! the live (non-`_XBOX`) build has no destructor at all (the `vector<>`
//! members drop themselves), matching Rust's ordinary `Drop`. No stub for any
//! of these (porting-rules §20).
//!
//! Dropped, `_G2_LISTEN_SERVER_OPT`-only surface (OFF, `G2SV-D4`):
//! `CopyBoneCache` (`:578-583`, a free fn guarded by the same macro as the
//! class's `entityNum`/`g2ClientAttachments` surface) — no stub.
//!
//! Dropped, dead code even in the live build: the `#if 0` rag-smoothing
//! branch inside `SmoothLow` (`:274-312`) is permanently compiled out
//! (literal `#if 0`, not a build-config macro) — its `mod`/`rootBoneList`
//! reads (`G2_Find_Bone_ByNum`) never run; folded into `smooth_low`'s doc
//! comment, not a separate stub.

use core::ffi::c_void;

use mp_host_interface::EngineHost;
use mp_qshared::shared::{mdxaBone_t, qhandle_t, VectorNormalize};

use crate::ghoul2_system::{BoneCacheId, Ghoul2System};
use crate::render::bone_transform::{g2_transform_bone, multiply_3x4_matrix};
use crate::shared::bone_info_t::boneInfo_t;

// `mdxaHeader_t`/`mdxaSkelOffsets_t`/`mdxaSkel_t` byte-arithmetic offsets, read
// off the opaque `*mut c_void` model block `EngineHost::model_mdxa` returns.
// `G2SV-D5` forbids naming those loader types in this crate, so the ctor/
// `SmoothLow` reach their fields by fixed offset instead of casting to the
// struct (Raven does `(byte *)header + sizeof(mdxaHeader_t) + offsets->offsets[i]`,
// `tr_ghoul2.cpp:416-424`). `MAX_QPATH == 64`, so, from `mdx_format.h`:
//   `mdxaHeader_t` (`:351-371`): `numBones` @84, total size 100.
//   `mdxaSkel_t`   (`:388-396`): `parent` @68, `BasePoseMat` @72, `BasePoseMatInv` @120.
// Source: `oracle/codemp/renderer/mdx_format.h:351-396`
const MDXA_HEADER_SIZE: usize = 100;
const MDXA_HEADER_NUM_BONES_OFF: usize = 84;
const MDXA_SKEL_PARENT_OFF: usize = 68;
const MDXA_SKEL_BASE_POSE_MAT_OFF: usize = 72;
const MDXA_SKEL_BASE_POSE_MAT_INV_OFF: usize = 120;

/// Read the `mdxaHeader_t::numBones` field out of the opaque model block.
///
/// # Safety
/// `header` must be the live `.gla` block `EngineHost::model_mdxa` returned
/// (non-NULL, ≥ `sizeof(mdxaHeader_t)` bytes).
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:403`
unsafe fn mdxa_num_bones(header: *const c_void) -> i32 {
    header
        .cast::<u8>()
        .add(MDXA_HEADER_NUM_BONES_OFF)
        .cast::<i32>()
        .read_unaligned()
}

/// Base pointer of bone `index`'s `mdxaSkel_t`, via the `mdxaSkelOffsets_t`
/// table at `header + sizeof(mdxaHeader_t)` (`tr_ghoul2.cpp:416-421`).
///
/// # Safety
/// As [`mdxa_num_bones`], with `index` in `0..numBones`.
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:416-421`
unsafe fn mdxa_skel(header: *const c_void, index: i32) -> *const u8 {
    let base = header.cast::<u8>();
    let offset = base
        .add(MDXA_HEADER_SIZE + index as usize * core::mem::size_of::<i32>())
        .cast::<i32>()
        .read_unaligned();
    base.add(MDXA_HEADER_SIZE + offset as usize)
}

/// Raven `struct SBoneCalc` (`tr_ghoul2.cpp:192-201`) — the frame/lerp inputs
/// for one bone, copied down from parent to child in `EvalLow` before
/// `G2_TransformBone` runs.
///
/// Raven: (none).
/// Type definition source: `oracle/codemp/renderer/tr_ghoul2.cpp:192-201`
// `vector<SBoneCalc>::resize` value-initializes each element (POD, no ctor) —
// all-zero / `blendMode=false`, matching `#[derive(Default)]`.
#[derive(Clone, Copy, Default)]
pub struct SBoneCalc {
    /// Raven `int newFrame`.
    pub new_frame: i32,
    /// Raven `int currentFrame`.
    pub current_frame: i32,
    /// Raven `float backlerp`.
    pub backlerp: f32,
    /// Raven `float blendFrame`.
    pub blend_frame: f32,
    /// Raven `int blendOldFrame`.
    pub blend_old_frame: i32,
    /// Raven `bool blendMode`.
    pub blend_mode: bool,
    /// Raven `float blendLerp`.
    pub blend_lerp: f32,
}

/// Raven `class CTransformBone` (`tr_ghoul2.cpp:166-190`) — the evaluated
/// bone matrix + parent link + `touch` memoization stamps.
///
/// Frozen verbatim in `docs/subsystems/ghoul2-server.md` `## Seam definition`.
/// Type definition source: `oracle/codemp/renderer/tr_ghoul2.cpp:166-190`
#[derive(Clone, Copy)]
pub struct CTransformBone {
    /// Raven `mdxaBone_t boneMatrix` — final matrix.
    pub bone_matrix: mdxaBone_t,
    /// Raven `int parent` — only set once, from `mdxaSkel_t` at ctor time.
    pub parent: i32,
    /// Raven `int touch` — recalculation memo stamp.
    pub touch: i32,
    /// Raven `int touchRender` (`//rww - RAGDOLL_BEGIN`) — render-traversal
    /// memo stamp, distinct from `touch`.
    pub touch_render: i32,
}

impl Default for CTransformBone {
    // Raven's `CTransformBone()` ctor sets only `touch`/`touchRender` to 0 and
    // leaves `boneMatrix`/`parent` uninitialized (`tr_ghoul2.cpp:182-188`); the
    // ctor loop then sets every `mFinalBones[i].parent`, and `boneMatrix` is
    // always written by `Eval` before any read. Zeroing both here is the one
    // defined Rust behavior for that Raven uninitialized-read window (F19); the
    // seam is unaffected — `mSmoothBones[i].parent` is never read.
    fn default() -> Self {
        CTransformBone {
            bone_matrix: mdxaBone_t {
                matrix: [[0.0; 4]; 3],
            },
            parent: 0,
            touch: 0,
            touch_render: 0,
        }
    }
}

/// Raven `class CBoneCache` (`tr_ghoul2.cpp:206-567`) — the per-model bone
/// evaluation cache built by `G2_ConstructGhoulSkeleton`/
/// `G2_TransformGhoulBones` (`render/skeleton.rs`) and owned by
/// `Ghoul2System.bone_caches` (`G2SV-D9`) via `BoneCacheId`, replacing Raven's
/// raw `CBoneCache *mBoneCache` per `CGhoul2Info` (`ghoul2_shared.h:265`).
///
/// Raven: (no class-level comment).
/// Type definition source: `oracle/codemp/renderer/tr_ghoul2.cpp:206-567`
pub struct CBoneCache {
    /// Raven `int frameSize` — `//can be deleted in new G2 format`; set to
    /// `0` by `G2_TransformGhoulBones` (`:2217`), never read live (only
    /// inside commented-out byte-arithmetic, `:1615-1687`).
    pub frame_size: i32,
    /// Raven `const mdxaHeader_t *header` — the loader's parsed `.gla` block.
    /// `*mut c_void` per `G2SV-D5`: this crate never names `mdxaHeader_t`;
    /// sourced from `EngineHost::model_mdxa` at ctor time.
    pub header: *mut c_void,
    /// Raven `const model_t *mod` — renamed (`mod` is a Rust keyword) and
    /// retyped to the `qhandle_t` the ctor received: `G2SV-D5` forbids naming
    /// `model_t` in this crate, and the live (non-`#if 0`) code path never
    /// dereferences the pointer itself, only passes it through.
    pub model: qhandle_t,
    /// Raven `vector<SBoneCalc> mBones` — per-bone frame/lerp inputs.
    pub bones: Vec<SBoneCalc>,
    /// Raven `vector<CTransformBone> mFinalBones` (`_XBOX` off, the live arm,
    /// `:364`) — the evaluated matrix + parent + `touch` stamp.
    pub final_bones: Vec<CTransformBone>,
    /// Raven `vector<CTransformBone> mSmoothBones` (`:366`) — render-smoothing
    /// history.
    pub smooth_bones: Vec<CTransformBone>,
    /// Raven `boneInfo_v *rootBoneList` — scratch pointer set fresh each
    /// `G2_TransformGhoulBones` call (`:2219`) into the owning
    /// `CGhoul2Info.blist`; read by `EvalLow`'s dead `#if 0` branch and by
    /// `G2_TransformBone` (`render/bone_transform.rs`). Kept as a raw pointer
    /// (internal-only, no ABI surface — porting-rules §A1): the arena split
    /// between `Ghoul2System.bone_caches` and `CGhoul2Info.blist` makes a safe
    /// borrow here a lifetime question the doc leaves to the per-file body.
    pub root_bone_list: *mut Vec<boneInfo_t>,
    /// Raven `mdxaBone_t rootMatrix` — copied in by `G2_TransformGhoulBones`
    /// (`:2220`), read by the transform chain.
    pub root_matrix: mdxaBone_t,
    /// Raven `int incomingTime`.
    pub incoming_time: i32,
    /// Raven `int mCurrentTouch` — starts at `3` (`:426`).
    pub current_touch: i32,
    /// Raven `int mCurrentTouchRender` (`//rww - RAGDOLL_BEGIN`).
    pub current_touch_render: i32,
    /// Raven `int mLastTouch` — starts at `2` (`:428`).
    pub last_touch: i32,
    /// Raven `int mLastLastTouch` — starts at `1` (`:429`).
    pub last_last_touch: i32,
    /// Raven `bool mSmoothingActive`.
    pub smoothing_active: bool,
    /// Raven `bool mUnsquash`.
    pub unsquash: bool,
    /// Raven `float mSmoothFactor`.
    pub smooth_factor: f32,
}

impl CBoneCache {
    /// Raven `CBoneCache::CBoneCache(const model_t *amod, const mdxaHeader_t
    /// *aheader)` (`:390-431`): sizes `mBones`/`mFinalBones`/`mSmoothBones` to
    /// `header->numBones`, seeds each `mFinalBones[i].parent` from the
    /// model's `mdxaSkel_t` (byte arithmetic off the header, an
    /// `mdxaSkelOffsets_t` at `header + sizeof(mdxaHeader_t)`), and starts the
    /// touch generation at `mCurrentTouch=3`/`mLastTouch=2`/
    /// `mLastLastTouch=1`. `header` is read via `EngineHost::model_mdxa`
    /// (`G2SV-D5`, ruling 36); the `_XBOX` `mNumBones`/raw-array arm is
    /// dropped (`_XBOX` off).
    ///
    /// Frozen verbatim in `docs/subsystems/ghoul2-server.md` `## Seam
    /// definition`.
    /// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:390-431`
    pub fn new(host: &mut impl EngineHost, a_mod: qhandle_t) -> Self {
        // Raven reads `header` from `R_GetModelByHandle(amod)->mdxa`; the port
        // reaches the same loader block over `EngineHost::model_mdxa` (`G2SV-D5`,
        // ruling 36). `assert(amod); assert(aheader)` (`:394-395`) — debug-only
        // (NDEBUG in the frozen build), kept as `debug_assert`.
        let header = host.model_mdxa(a_mod);
        debug_assert!(!header.is_null(), "CBoneCache::new: null mdxa header");

        // SAFETY: `header` is the live `.gla` block from `model_mdxa`.
        let num_bones = unsafe { mdxa_num_bones(header) };
        let n = num_bones as usize;

        let bones = vec![SBoneCalc::default(); n];
        let mut final_bones = vec![CTransformBone::default(); n];
        let smooth_bones = vec![CTransformBone::default(); n];

        // Seed each bone's parent from the model's `mdxaSkel_t` (`:419-425`).
        for (i, bone) in final_bones.iter_mut().enumerate() {
            // SAFETY: `header` valid, `i < numBones`.
            let skel = unsafe { mdxa_skel(header, i as i32) };
            bone.parent = unsafe {
                skel.add(MDXA_SKEL_PARENT_OFF)
                    .cast::<i32>()
                    .read_unaligned()
            };
        }

        CBoneCache {
            frame_size: 0,
            header,
            model: a_mod,
            bones,
            final_bones,
            smooth_bones,
            root_bone_list: core::ptr::null_mut(),
            root_matrix: mdxaBone_t {
                matrix: [[0.0; 4]; 3],
            },
            incoming_time: 0,
            // Raven `mCurrentTouch=3`, `mLastTouch=2`, `mLastLastTouch=1` (`:426-429`).
            current_touch: 3,
            current_touch_render: 0,
            last_touch: 2,
            last_last_touch: 1,
            smoothing_active: false,
            unsquash: false,
            smooth_factor: 0.0,
        }
    }

    /// Raven `SBoneCalc &CBoneCache::Root()` (`:441-445`) — `mBones[0]`, the
    /// traversal root; asserts `mBones` is non-empty. Called by
    /// `G2_TransformGhoulBones` (`render/skeleton.rs`, `:2223`,
    /// `TB=ghoul2.mBoneCache->Root()`).
    ///
    /// Not enumerated in the doc's `render/bone_cache.rs` roster summary line
    /// (doc/oracle mismatch — reported, not improvised around); kept here
    /// because `G2_TransformGhoulBones`, a doc-ported function, calls it, and
    /// a missing method would block that porter.
    /// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:441-445`
    pub fn root(&mut self) -> &mut SBoneCalc {
        debug_assert!(!self.bones.is_empty());
        &mut self.bones[0]
    }

    /// Raven `void CBoneCache::EvalLow(int index)` (`:236-265`, private) —
    /// the memoized-recursion core: recurses to the parent, copies its
    /// `SBoneCalc` down, calls the free function `G2_TransformBone(index,
    /// *this)` (`render/bone_transform.rs`), and stamps
    /// `mFinalBones[index].touch=mCurrentTouch`. The `_XBOX`
    /// `SetRenderMatrix` call is dropped (`_XBOX` off).
    ///
    /// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:236-265`
    fn eval_low(&mut self, index: i32) {
        let idx = index as usize;
        debug_assert!(index >= 0 && idx < self.bones.len());
        if self.final_bones[idx].touch != self.current_touch {
            // need to evaluate the bone
            let parent = self.final_bones[idx].parent;
            debug_assert!(
                (parent >= 0 && (parent as usize) < self.final_bones.len())
                    || (index == 0 && parent == -1)
            );
            if parent >= 0 {
                self.eval_low(parent); // make sure parent is evaluated
                                       // Copy the parent's frame/lerp inputs down (all seven fields).
                self.bones[idx] = self.bones[parent as usize];
            }
            g2_transform_bone(self, index);
            self.final_bones[idx].touch = self.current_touch;
        }
    }

    /// Raven `void CBoneCache::SmoothLow(int index)` (`:267-351`, private) —
    /// render smoothing: LERPs `mSmoothBones[index]` toward
    /// `mFinalBones[index]` (or copies straight through on the first touch),
    /// then applies the basepose un-scale/rescale (`Multiply_3x4Matrix`
    /// twice against `mdxaSkel_t::BasePoseMat`/`BasePoseMatInv`, read via the
    /// same `EngineHost::model_mdxa` block as the ctor). The `#if 0`
    /// rag-smoothing branch (`:274-312`) is permanently dead code (not a
    /// build macro) and is not transcribed; the `_DEBUG _isnan` asserts are
    /// dropped (F19).
    ///
    /// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:267-351`
    fn smooth_low(&mut self, index: i32) {
        let idx = index as usize;
        if self.smooth_bones[idx].touch == self.last_touch {
            // LERP the 12 matrix elements toward the fresh final matrix. (The
            // `#if 0` per-bone rag-smoothing branch, `:274-312`, is permanently
            // dead code — not transcribed.)
            let smooth_factor = self.smooth_factor;
            for i in 0..3 {
                for j in 0..4 {
                    let old_m = self.smooth_bones[idx].bone_matrix.matrix[i][j];
                    let new_m = self.final_bones[idx].bone_matrix.matrix[i][j];
                    self.smooth_bones[idx].bone_matrix.matrix[i][j] =
                        smooth_factor * (old_m - new_m) + new_m;
                }
            }
        } else {
            // First touch: copy the final matrix straight through.
            self.smooth_bones[idx].bone_matrix = self.final_bones[idx].bone_matrix;
        }

        // Un-scale then rescale by the basepose to remove squash/stretch. The
        // basepose matrices come from the model's `mdxaSkel_t`, read off the
        // same opaque `header` block as the ctor (`G2SV-D5`).
        // SAFETY: `header` is the live `.gla` block; `index` in `0..numBones`.
        let skel = unsafe { mdxa_skel(self.header, index) };
        let base_pose_mat: mdxaBone_t = unsafe {
            skel.add(MDXA_SKEL_BASE_POSE_MAT_OFF)
                .cast::<mdxaBone_t>()
                .read_unaligned()
        };
        let base_pose_mat_inv: mdxaBone_t = unsafe {
            skel.add(MDXA_SKEL_BASE_POSE_MAT_INV_OFF)
                .cast::<mdxaBone_t>()
                .read_unaligned()
        };

        let mut temp_matrix = mdxaBone_t {
            matrix: [[0.0; 4]; 3],
        };
        multiply_3x4_matrix(
            &mut temp_matrix,
            &self.smooth_bones[idx].bone_matrix,
            &base_pose_mat,
        );
        // `maxl = VectorLength(&skel->BasePoseMat.matrix[0][0])` — a macro over
        // the row's first three components; `sqrt` is libm double rounded to
        // float, matching `VectorNormalize` below.
        let r = base_pose_mat.matrix[0];
        let maxl = ((r[0] * r[0] + r[1] * r[1] + r[2] * r[2]) as f64).sqrt() as f32;

        // `VectorNormalize` each of the three rows' first three components.
        for row in &mut temp_matrix.matrix {
            let mut v = [row[0], row[1], row[2]];
            VectorNormalize(&mut v);
            row[0] = v[0];
            row[1] = v[1];
            row[2] = v[2];
        }
        // `VectorScale( row, maxl, row )` — first three components.
        for row in &mut temp_matrix.matrix {
            row[0] *= maxl;
            row[1] *= maxl;
            row[2] *= maxl;
        }
        multiply_3x4_matrix(
            &mut self.smooth_bones[idx].bone_matrix,
            &temp_matrix,
            &base_pose_mat_inv,
        );
        self.smooth_bones[idx].touch = self.current_touch;
        // The `_DEBUG` `_isnan` asserts (`:342-350`) are dropped (F19).
    }

    /// Raven `const mdxaBone_t &CBoneCache::EvalUnsmooth(int index)`
    /// (`:446-454`) — `EvalLow` then returns the smoothed matrix if smoothing
    /// is active and has been touched, else the plain final matrix.
    ///
    /// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:446-454`
    pub fn eval_unsmooth(&mut self, index: i32) -> mdxaBone_t {
        self.eval_low(index);
        let idx = index as usize;
        if self.smoothing_active && self.smooth_bones[idx].touch != 0 {
            return self.smooth_bones[idx].bone_matrix;
        }
        self.final_bones[idx].bone_matrix
    }

    /// Raven `const mdxaBone_t &CBoneCache::Eval(int index)` (`:455-518`) —
    /// the live (SOF2-style) body: `EvalLow` only when
    /// `touch!=mCurrentTouch`, then returns `mFinalBones[index].boneMatrix`.
    /// The commented-out smoothing-blend body above it (`:457-509`) never
    /// compiles (`/* ... */`).
    ///
    /// Frozen verbatim in `docs/subsystems/ghoul2-server.md` `## Seam
    /// definition`.
    /// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:455-518`
    pub fn eval(&mut self, index: i32) -> mdxaBone_t {
        // The live SOF2-style body; the commented-out smoothing-blend body above
        // it (`:457-509`) never compiles.
        let idx = index as usize;
        debug_assert!(index >= 0 && idx < self.bones.len());
        if self.final_bones[idx].touch != self.current_touch {
            self.eval_low(index);
        }
        self.final_bones[idx].bone_matrix
    }

    /// Raven `const inline mdxaBone_t &CBoneCache::EvalRender(int index)`
    /// (`:520-537`) — stamps `touchRender` then `EvalLow`s if unevaluated;
    /// `SmoothLow`s (memoized by `touch`) and returns the smoothed matrix
    /// when smoothing is active, else the plain final matrix.
    ///
    /// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:520-537`
    pub fn eval_render(&mut self, index: i32) -> mdxaBone_t {
        let idx = index as usize;
        debug_assert!(index >= 0 && idx < self.bones.len());
        if self.final_bones[idx].touch != self.current_touch {
            self.final_bones[idx].touch_render = self.current_touch_render;
            self.eval_low(index);
        }
        if self.smoothing_active {
            if self.smooth_bones[idx].touch != self.current_touch {
                self.smooth_low(index);
            }
            return self.smooth_bones[idx].bone_matrix;
        }
        self.final_bones[idx].bone_matrix
    }

    /// Raven `bool CBoneCache::WasRendered(int index)` (`:540-544`) —
    /// `mFinalBones[index].touchRender==mCurrentTouchRender`.
    ///
    /// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:540-544`
    pub fn was_rendered(&self, index: i32) -> bool {
        let idx = index as usize;
        debug_assert!(index >= 0 && idx < self.bones.len());
        self.final_bones[idx].touch_render == self.current_touch_render
    }

    /// Raven `int CBoneCache::GetParent(int index)` (`:545-553`) — `-1` for
    /// the root (`index==0`), else `mFinalBones[index].parent`.
    ///
    /// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:545-553`
    pub fn get_parent(&self, index: i32) -> i32 {
        if index == 0 {
            return -1;
        }
        let idx = index as usize;
        debug_assert!(index >= 0 && idx < self.bones.len());
        self.final_bones[idx].parent
    }
}

/// Raven `const mdxaBone_t &EvalBoneCache(int index, CBoneCache *boneCache)`
/// (`:585-589`) — asserts non-null then forwards to `boneCache->Eval(index)`.
/// The arena lookup (`g2.bone_caches.get_mut(cache)`) replaces the raw
/// pointer null-check (`G2SV-D9`).
///
/// Frozen verbatim in `docs/subsystems/ghoul2-server.md` `## Seam definition`.
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:585-589`
pub fn eval_bone_cache(g2: &mut Ghoul2System, cache: BoneCacheId, index: i32) -> mdxaBone_t {
    // Raven `assert(boneCache); return boneCache->Eval(index);`. The arena
    // lookup replaces the raw-pointer null-check: a stale/absent handle (Raven's
    // NULL `boneCache`) panics here, the one defined behavior for Raven's
    // assert-then-null-deref (`G2SV-D9`).
    g2.bone_caches
        .get_mut(cache)
        .expect("EvalBoneCache: stale/null bone-cache handle")
        .eval(index)
}

/// Raven `void RemoveBoneCache(CBoneCache *boneCache)` (`:569-576`) —
/// `delete`s the cache (plus the `_FULL_G2_LEAK_CHECKING` counter decrement,
/// dropped per §F20/no parity surface). Ported as the arena's `remove`
/// (`Ghoul2System.bone_caches.remove`, `G2SV-D9`); called both from
/// `Ghoul2System::delete_low` (`ghoul2_system.rs`, `G2SV-D13`(a)) and directly
/// from the `RemoveGhoul2Model(s)`/`CopyGhoul2Instance` call sites
/// (`G2_API.cpp:821,908,2311`).
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:569-576`
pub fn remove_bone_cache(g2: &mut Ghoul2System, cache: BoneCacheId) {
    // Raven `delete boneCache;` (plus the `_FULL_G2_LEAK_CHECKING` counter
    // decrement, dropped — no parity surface, §F20). The arena's `remove` frees
    // the owned cache and bumps the slot generation (`G2SV-D9`).
    g2.bone_caches.remove(cache);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `CBoneCache` with `n` bones, sidestepping the model-memory ctor so the
    /// touch-stamp read paths (`get_parent`/`was_rendered`/`root`) can be tested
    /// without an `EngineHost` or the not-yet-filled transform siblings.
    fn bare_cache(n: usize) -> CBoneCache {
        CBoneCache {
            frame_size: 0,
            header: core::ptr::null_mut(),
            model: 0,
            bones: vec![SBoneCalc::default(); n],
            final_bones: vec![CTransformBone::default(); n],
            smooth_bones: vec![CTransformBone::default(); n],
            root_bone_list: core::ptr::null_mut(),
            root_matrix: mdxaBone_t {
                matrix: [[0.0; 4]; 3],
            },
            incoming_time: 0,
            current_touch: 3,
            current_touch_render: 0,
            last_touch: 2,
            last_last_touch: 1,
            smoothing_active: false,
            unsquash: false,
            smooth_factor: 0.0,
        }
    }

    #[test]
    fn get_parent_roots_index_zero() {
        // Root always returns -1 regardless of the stored parent (`:547-549`).
        let mut c = bare_cache(3);
        c.final_bones[0].parent = 99;
        c.final_bones[1].parent = 0;
        c.final_bones[2].parent = 1;
        assert_eq!(c.get_parent(0), -1);
        assert_eq!(c.get_parent(1), 0);
        assert_eq!(c.get_parent(2), 1);
    }

    #[test]
    fn was_rendered_matches_current_touch_render() {
        // `mFinalBones[i].touchRender == mCurrentTouchRender` (`:543`).
        let mut c = bare_cache(2);
        c.current_touch_render = 7;
        c.final_bones[0].touch_render = 7;
        c.final_bones[1].touch_render = 6;
        assert!(c.was_rendered(0));
        assert!(!c.was_rendered(1));
    }

    #[test]
    fn root_is_first_bone() {
        // `Root()` is `mBones[0]` (`:441-445`).
        let mut c = bare_cache(2);
        c.root().new_frame = 42;
        assert_eq!(c.bones[0].new_frame, 42);
    }
}
