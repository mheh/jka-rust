#![allow(non_camel_case_types, non_snake_case)]

//! Raven `CGhoul2Info` (`ghoul2_shared.h:240-312`) — the per-instance Ghoul2
//! model slot, `vector`-held inside `Ghoul2InfoArray::mInfos` (`info_array.rs`).
//! A §F idiomatic reimplementation, **not** an ABI-frozen layout (porting-rules
//! §F17: intrusive STL members -> owned `Vec`/`String`); `mSlist`/`mBltlist`/
//! `mBlist` become owned `Vec`s, so this struct is not `#[repr(C)]` and carries
//! no size/offset asserts. Distinct from the already-ported handle wrapper
//! `CGhoul2Info_v` (`shared/cghoul2_info_v.rs`), which merely holds an arena
//! index into a `Vec<CGhoul2Info>`.
//!
//! `entityNum` (`ghoul2_shared.h:278`, `_G2_LISTEN_SERVER_OPT`) is dropped: that
//! macro is OFF in the WinDed DEDICATED build (`G2SV-D4`), so the field never
//! compiles in.

use crate::ghoul2_system::BoneCacheId;
use crate::shared::bolt_info_t::boltInfo_t;
use crate::shared::bone_info_t::boneInfo_t;
use crate::shared::surface_info_t::surfaceInfo_t;
use mp_host_interface::mdx::mdxa::MdxaRef;
use mp_host_interface::mdx::mdxm::MdxmRef;
use mp_qshared::shared::qhandle_t;

/// Raven `CGhoul2Info` — one Ghoul2 model instance's full state.
///
/// Raven: (no class-level comment).
/// Type definition source: `oracle/codemp/ghoul2/ghoul2_shared.h:240-312`
///
/// `Clone` realizes Raven's implicit copy constructor: `DeepCopy` copies the
/// instance vector (`Array()=other.Array()`, `ghoul2_shared.h:385`) which
/// memberwise-copies each `CGhoul2Info` — its owned `Vec`s (`slist`/`bltlist`/
/// `blist`) deep-copy, its raw view pointers copy shallowly (re-validated by
/// `G2_SetupModelPointers`), exactly as the C++ default copy would.
#[derive(Clone)]
pub struct CGhoul2Info {
    /// Raven `surfaceInfo_v mSlist` — per-surface on/off + override list.
    pub slist: Vec<surfaceInfo_t>,
    /// Raven `boltInfo_v mBltlist` — per-bolt attachment list.
    pub bltlist: Vec<boltInfo_t>,
    /// Raven `boneInfo_v mBlist` — per-bone override/ragdoll list.
    pub blist: Vec<boneInfo_t>,

    // save from here (ghoul2_shared.h:246) —
    /// Raven `int mModelindex`.
    pub modelindex: i32,
    /// Raven `qhandle_t mCustomShader`.
    pub custom_shader: qhandle_t,
    /// Raven `qhandle_t mCustomSkin`.
    pub custom_skin: qhandle_t,
    /// Raven `int mModelBoltLink`.
    pub model_bolt_link: i32,
    /// Raven `int mSurfaceRoot`.
    pub surface_root: i32,
    /// Raven `int mLodBias`.
    pub lod_bias: i32,
    /// this contains the bolt index of the new origin for this model
    pub new_origin: i32,
    /// Raven `int mGoreSetTag` — `_G2_GORE` ON in the WinDed build (`G2SV-D5`),
    /// so this field always compiles in.
    pub gore_set_tag: i32,

    /// this and the next entries do NOT go across the network. They are for
    /// gameside access ONLY
    pub model: qhandle_t,
    /// Raven `char mFileName[MAX_QPATH]` — owned `String` (porting-rules §C9),
    /// not a fixed byte array.
    pub file_name: String,
    /// Raven `int mAnimFrameDefault`.
    pub anim_frame_default: i32,
    /// Raven `int mSkelFrameNum`.
    pub skel_frame_num: i32,
    /// Raven `int mMeshFrameNum`.
    pub mesh_frame_num: i32,
    /// used for determining whether to do full collision detection against
    /// this object
    pub flags: i32,
    // to here (end of the save-serialized middle band, ghoul2_shared.h:263)
    /// used to create an array of pointers to transformed verts per surface
    /// for collision detection. Raven `int *mTransformedVertsArray`
    /// (`Z_Malloc`-owned raw array, porting-rules §C9 -> owned `Vec`); `None`
    /// is Raven's null (unallocated).
    pub transformed_verts_array: Option<Vec<i32>>,
    /// Raven `CBoneCache *mBoneCache` (`ghoul2_shared.h:265`) — replaced by a
    /// generational handle into `Ghoul2System.bone_caches` (§B5, `G2SV-D9`); no
    /// raw pointer escapes the ABI seam.
    pub bone_cache: Option<BoneCacheId>,
    /// Raven `int mSkin`.
    pub skin: i32,

    // these occasionally are not valid (like after a vid_restart)
    // call the questionably efficient G2_SetupModelPointers(this) to insure validity
    /// all the below are proper and valid
    pub valid: bool,
    /// Raven `const model_s *currentModel` — opaque (never named as an
    /// `mp_renderer` type, same rationale as `mdxaHeader_t`/`mdxaSkel_t`,
    /// `G2SV-D5`); resolved/validated by `G2_SetupModelPointers` (`misc.rs`).
    /// Stored as the loader-block ref (`None` ≡ the null pointer); the ref's
    /// `'static` contract (both `parsed` and `view`) is revalidated at each
    /// `G2_SetupModelPointers`.
    pub current_model: Option<MdxmRef<'static>>,
    /// Raven `int currentModelSize`.
    pub current_model_size: i32,
    /// Raven `const model_s *animModel` — opaque, see `current_model`.
    pub anim_model: Option<MdxaRef<'static>>,
    /// Raven `int currentAnimModelSize`.
    pub current_anim_model_size: i32,
    /// Raven `const mdxaHeader_t *aHeader` — opaque (`G2SV-D5`: `mdxaHeader_t`
    /// is never named as a Rust type here); read via `EngineHost::model_mdxa`.
    pub a_header: Option<MdxaRef<'static>>,
}

impl Default for CGhoul2Info {
    /// Raven `CGhoul2Info::CGhoul2Info()` default ctor initializer list.
    /// Source: `oracle/codemp/ghoul2/ghoul2_shared.h:281-311`
    fn default() -> Self {
        CGhoul2Info {
            slist: Vec::new(),
            bltlist: Vec::new(),
            blist: Vec::new(),
            modelindex: -1,
            custom_shader: 0,
            custom_skin: 0,
            model_bolt_link: 0,
            surface_root: 0,
            lod_bias: 0,
            new_origin: -1,
            gore_set_tag: 0,
            model: 0,
            file_name: String::new(),
            anim_frame_default: 0,
            skel_frame_num: -1,
            mesh_frame_num: -1,
            flags: 0,
            transformed_verts_array: None,
            bone_cache: None,
            skin: 0,
            valid: false,
            current_model: None,
            current_model_size: 0,
            anim_model: None,
            current_anim_model_size: 0,
            a_header: None,
        }
    }
}
