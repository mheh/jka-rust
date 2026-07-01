//! MP executable common substrate corresponding to Raven's `codemp/qcommon` tree.

pub mod aas_areainfo;
pub mod bot_goal;
pub mod collision_record;
pub mod entity_state;
pub mod failed_edge;
pub mod game_item;
pub mod parms;
pub mod platform;
pub mod player_state;
pub mod qtime;
pub mod saber;
pub mod shared_ragdoll_params;
pub mod shared_ragdoll_update_params;
pub mod shared_set_bone_ik_state_params;
pub mod siege_pers;
pub mod tags;
pub mod usercmd;

pub use aas_areainfo::aas_areainfo_t;
pub use bot_goal::bot_goal_t;
pub use entity_state::entityState_t;
pub use failed_edge::failedEdge_t;
pub use game_item::{gitem_t, itemType_t, MAX_ITEM_MODELS};
pub use parms::{parms_t, MAX_PARMS, MAX_PARM_STRING_LENGTH};
pub use player_state::playerState_t;
pub use qtime::qtime_t;
pub use saber::{
    bladeInfo_t, saberInfo_t, saberTrail_t, saberType_t, saber_colors_t, saber_styles_t,
    MAX_BLADES, MAX_SABERS,
};
pub use shared_ragdoll_params::sharedRagDollParams_t;
pub use shared_ragdoll_update_params::sharedRagDollUpdateParams_t;
pub use shared_set_bone_ik_state_params::sharedSetBoneIKStateParams_t;
pub use siege_pers::siegePers_t;
pub use usercmd::usercmd_t;
