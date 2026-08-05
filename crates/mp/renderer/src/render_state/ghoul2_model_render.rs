//! `Ghoul2ModelRender` - one render-visible model of a Ghoul2 entity, snapshotted at scene-add.

use mp_engine_ghoul2::shared::bolt_info_t::boltInfo_t;
use mp_engine_ghoul2::shared::surface_info_t::surfaceInfo_t;
use mp_qshared::shared::{mdxaBone_t, qhandle_t};

/// One render-visible model of a Ghoul2 entity, snapshotted sim-side at scene-add.
/// It has no Raven counterpart: DEC-65 ruling 2 replaces the render side's reach into the live `CGhoul2Info` with these values.
/// Every field is POD or owned, so the type is `Send + Sync` without unsafe.
pub struct Ghoul2ModelRender {
    /// The instance's registered model handle, which the walk resolves against the published registry.
    pub model: qhandle_t,
    /// `mCustomSkin` on the instance.
    pub custom_skin: qhandle_t,
    /// `mSkin` on the instance.
    pub skin: qhandle_t,
    /// `mLodBias` on the instance.
    pub lod_bias: i32,
    /// `mSurfaceRoot` on the instance.
    pub surface_root: i32,
    /// The instance's surface-override list, cloned at scene-add.
    pub slist: Vec<surfaceInfo_t>,
    /// The instance's bolt list, cloned at scene-add.
    pub bltlist: Vec<boltInfo_t>,
    /// The composed render matrix per bone, indexed by the global bone index `surface.bone_ref` yields.
    /// Empty when the instance has no built bone cache, and the walk then drops the model's surfaces.
    pub bones: Vec<mdxaBone_t>,
}
