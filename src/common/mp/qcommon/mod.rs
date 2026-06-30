//! MP executable common substrate corresponding to Raven's `codemp/qcommon` tree.

pub mod aas_areainfo;
pub mod collision_record;
pub mod platform;
pub mod qtime;
pub mod shared_ragdoll_params;
pub mod shared_set_bone_ik_state_params;
pub mod tags;
pub mod usercmd;

pub use aas_areainfo::aas_areainfo_t;
pub use qtime::qtime_t;
pub use shared_ragdoll_params::sharedRagDollParams_t;
pub use shared_set_bone_ik_state_params::sharedSetBoneIKStateParams_t;
pub use usercmd::usercmd_t;
