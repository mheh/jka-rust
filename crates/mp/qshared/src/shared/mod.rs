//! Globally-shared wire & math types (Raven `q_shared.h` scope): vec3, entityState, playerState, trace, usercmd.

#![allow(non_camel_case_types)]

pub mod add_electricity_arg;
pub mod addbezier_arg;
pub mod addpoly_arg;
pub mod addsprite_arg;
pub mod cbuf_exec;
pub mod collision;
pub mod effect_trail_arg;
pub mod effect_trail_vert;
pub mod game_state;
pub mod connstate;
pub mod ct_table;
pub mod cvar;
#[path = "e_status.rs"]
pub mod cinematic_status;
pub mod entity_shared;
pub mod error_parm;
pub mod flag_status;
pub mod force_powers;
pub mod force_reload;
pub mod fs_origin;
pub mod gen_cmds;
#[path = "ha_pref.rs"]
pub mod hunk_pref;
pub mod item_use_fail;
pub mod limits;
pub mod mark_fragment;
pub mod print_parm;
#[path = "qint64.rs"]
pub mod int64;
pub mod saber_block_type;
pub mod saber_blocked_type;
pub mod shared_eik_move_state;
pub mod shared_erag_effector;
pub mod shared_erag_phase;
pub mod sound_channel;
pub mod string_id_table;
pub mod trackchan;
pub mod vec3struct;
pub mod wl_e;
pub mod wpneighbor;
pub mod wpobject;
#[path = "fsMode_t.rs"]
pub mod file_mode;
#[path = "pc_token_t.rs"]
pub mod pc_token;
#[path = "sharedIKMoveParams_t.rs"]
pub mod shared_ik_move_params;
pub mod trajectory;

// Migration bridge: these tiers now live in dedicated crates. `jka` re-exports
// them so its remaining modules stay green until the monolith is dissolved.
pub use native_math::vector;
pub use native_platform::platform;

pub use add_electricity_arg::addElectricityArgStruct_t;
pub use addbezier_arg::addbezierArgStruct_t;
pub use addpoly_arg::addpolyArgStruct_t;
pub use addsprite_arg::addspriteArgStruct_t;
pub use cbuf_exec::cbufExec_t;
pub use collision::{cplane_t, CollisionRecord_t};
pub use effect_trail_arg::effectTrailArgStruct_t;
pub use effect_trail_vert::effectTrailVertStruct_t;
pub use game_state::{gameState_t, MAX_CONFIGSTRINGS, MAX_GAMESTATE_CHARS};
pub use connstate::connstate_t;
pub use ct_table::ct_table_t;
pub use cvar::{cvarHandle_t, vmCvar_t, MAX_CVAR_VALUE_STRING};
pub use cinematic_status::{
    e_status, FMV_EOF, FMV_IDLE, FMV_ID_BLT, FMV_ID_IDLE, FMV_ID_WAIT, FMV_LOOPED, FMV_PLAY,
};
pub use entity_shared::entityShared_t;
pub use error_parm::errorParm_t;
pub use file_mode::{fsMode_t, FS_APPEND, FS_APPEND_SYNC, FS_READ, FS_WRITE};
pub use flag_status::{
    flagStatus_t, FLAG_ATBASE, FLAG_DROPPED, FLAG_TAKEN, FLAG_TAKEN_BLUE, FLAG_TAKEN_RED,
};
pub use force_powers::{
    forcePowers_t, FP_ABSORB, FP_DRAIN, FP_FIRST, FP_GRIP, FP_HEAL, FP_LEVITATION, FP_LIGHTNING,
    FP_PROTECT, FP_PULL, FP_PUSH, FP_RAGE, FP_SABERTHROW, FP_SABER_DEFENSE, FP_SABER_OFFENSE,
    FP_SEE, FP_SPEED, FP_TEAM_FORCE, FP_TEAM_HEAL, FP_TELEPATHY, NUM_FORCE_POWERS,
};
pub use force_reload::ForceReload_e;
pub use fs_origin::fsOrigin_t;
pub use gen_cmds::genCmds_t;
pub use hunk_pref::ha_pref;
pub use item_use_fail::itemUseFail_t;
pub use limits::{MAX_CLIENTS, MAX_STRING_CHARS};
pub use mark_fragment::markFragment_t;
pub use pc_token::{pc_token_t, MAX_TOKENLENGTH};
pub use print_parm::printParm_t;
pub use int64::qint64;
pub use saber_block_type::saberBlockType_t;
pub use saber_blocked_type::saberBlockedType_t;
pub use shared_eik_move_state::sharedEIKMoveState;
pub use shared_erag_effector::sharedERagEffector;
pub use shared_erag_phase::sharedERagPhase;
pub use shared_ik_move_params::sharedIKMoveParams_t;
pub use sound_channel::{
    soundChannel_t, CHAN_AMBIENT, CHAN_ANNOUNCER, CHAN_AUTO, CHAN_BODY, CHAN_ITEM, CHAN_LESS_ATTEN,
    CHAN_LOCAL, CHAN_LOCAL_SOUND, CHAN_MENU1, CHAN_MUSIC, CHAN_VOICE, CHAN_VOICE_ATTEN,
    CHAN_VOICE_GLOBAL, CHAN_WEAPON,
};
pub use string_id_table::stringID_table_t;
pub use trackchan::trackchan_t;
pub use trajectory::{trType_t, trajectory_t};
pub use vec3struct::vec3struct_t;
pub use wl_e::WL_e;
pub use wpneighbor::wpneighbor_t;
pub use wpobject::{wpobject_t, MAX_NEIGHBOR_SIZE};
pub use vector::{vec2_t, vec3_t, vec3pair_t, vec4_t, vec5_t, vec_t};

// Migration bridge: cross-mode scalar/handle primitives now live in `native_types`.
pub use native_types::{
    clipHandle_t, fileHandle_t, mdxaBone_t, qboolean, qhandle_t, MAX_QPATH, QFALSE, QTRUE,
};
