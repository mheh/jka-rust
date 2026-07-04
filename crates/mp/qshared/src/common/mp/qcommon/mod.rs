//! MP executable common substrate corresponding to Raven's `codemp/qcommon` tree.

pub mod aas_areainfo;
pub mod b_set_t;
pub mod b_state_t;
pub mod bot_goal;
pub mod collision_record;
pub mod entity_state;
pub mod failed_edge;
pub mod game_export_t;
pub mod game_import_t;
pub mod game_item;
pub mod nav_debug_draw;
pub mod parms;
pub mod platform;
pub mod player_state;
pub mod pm_flags;
pub mod qtime;
pub mod saber;
pub mod shared_entity_t;
pub mod shared_ragdoll_params;
pub mod shared_ragdoll_update_params;
pub mod shared_set_bone_ik_state_params;
pub mod siege_pers;
pub mod t_g_icarus_getfloat;
pub mod t_g_icarus_getsetidforstring;
pub mod t_g_icarus_getstring;
pub mod t_g_icarus_gettag;
pub mod t_g_icarus_getvector;
pub mod t_g_icarus_kill;
pub mod t_g_icarus_lerp2_angles;
pub mod t_g_icarus_lerp2_end;
pub mod t_g_icarus_lerp2_origin;
pub mod t_g_icarus_lerp2_pos;
pub mod t_g_icarus_lerp2_start;
pub mod t_g_icarus_play;
pub mod t_g_icarus_playsound;
pub mod t_g_icarus_remove;
pub mod t_g_icarus_set;
pub mod t_g_icarus_soundindex;
pub mod t_g_icarus_use;
pub mod tags;
pub mod task_id_t;
pub mod usercmd;
pub mod usercmd_button;

pub use aas_areainfo::aas_areainfo_t;
pub use bot_goal::bot_goal_t;
pub use entity_state::entityState_t;
pub use failed_edge::failedEdge_t;
pub use game_item::{gitem_t, itemType_t, MAX_ITEM_MODELS};
pub use parms::{parms_t, MAX_PARMS, MAX_PARM_STRING_LENGTH};
pub use player_state::playerState_t;
pub use pm_flags::{
    PMF_ALL_TIMES, PMF_BACKWARDS_JUMP, PMF_BACKWARDS_RUN, PMF_DUCKED, PMF_FIX_MINS, PMF_FOLLOW,
    PMF_JUMP_HELD, PMF_RESPAWNED, PMF_ROLLING, PMF_SCOREBOARD, PMF_STUCK_TO_WALL,
    PMF_TIME_KNOCKBACK, PMF_TIME_LAND, PMF_TIME_WATERJUMP, PMF_UPDATE_ANIM, PMF_USE_ITEM_HELD,
};
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
