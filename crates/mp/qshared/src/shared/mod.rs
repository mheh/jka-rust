//! Globally-shared wire & math types (Raven `q_shared.h` scope): vec3, entityState, playerState, trace, usercmd.

#![allow(non_camel_case_types)]

pub mod add_electricity_arg;
pub mod addbezier_arg;
pub mod addpoly_arg;
pub mod addsprite_arg;
pub mod build_ident;
pub mod cbuf_exec;
#[path = "e_status.rs"]
pub mod cinematic_status;
pub mod collision;
// S5-5: canonical home for the `QSharedScratch`-threaded `COM_Parse*` family
// (bg/game import the module path explicitly — no crate-root re-export, so it
// does not collide with the `static mut` `COM_*` twins in `q_string.rs`).
pub mod com_parse;
pub mod connstate;
pub mod ct_table;
pub mod cvar;
pub mod effect_trail_arg;
pub mod effect_trail_vert;
pub mod entity_shared;
pub mod error_parm;
#[path = "fsMode_t.rs"]
pub mod file_mode;
pub mod flag_status;
pub mod force_powers;
pub mod force_reload;
pub mod fs_origin;
pub mod game_state;
pub mod gen_cmds;
#[path = "ha_pref.rs"]
pub mod hunk_pref;
#[path = "qint64.rs"]
pub mod int64;
pub mod item_use_fail;
pub mod limits;
pub mod mark_fragment;
#[path = "pc_token_t.rs"]
pub mod pc_token;
pub mod print_parm;
pub mod q_color;
pub mod q_format;
pub mod q_math;
pub mod q_math_rand;
pub mod q_string;
pub mod saber_block_type;
pub mod saber_blocked_type;
pub mod shared_eik_move_state;
pub mod shared_erag_effector;
pub mod shared_erag_phase;
#[path = "sharedIKMoveParams_t.rs"]
pub mod shared_ik_move_params;
pub mod sound_channel;
pub mod string_id_table;
pub mod surface_flags;
pub mod swap;
pub mod trackchan;
pub mod trajectory;
pub mod vec3struct;
pub mod wl_e;
pub mod world_size;
pub mod wpneighbor;
pub mod wpobject;

// Migration bridge: these tiers now live in dedicated crates. `jka` re-exports
// them so its remaining modules stay green until the monolith is dissolved.
pub use native_math::orientation::orientation_t;
pub use native_math::vector;
pub use native_platform::platform;

pub use add_electricity_arg::addElectricityArgStruct_t;
pub use addbezier_arg::addbezierArgStruct_t;
pub use addpoly_arg::addpolyArgStruct_t;
pub use addsprite_arg::addspriteArgStruct_t;
pub use build_ident::{CPUSTRING, MAC_STATIC, PATH_SEP, Q3_VERSION, QDECL};
pub use cbuf_exec::cbufExec_t;
pub use cinematic_status::{
    e_status, FMV_EOF, FMV_IDLE, FMV_ID_BLT, FMV_ID_IDLE, FMV_ID_WAIT, FMV_LOOPED, FMV_PLAY,
};
pub use collision::{cplane_t, CollisionRecord_t, PLANE_X, PLANE_Y, PLANE_Z};
pub use connstate::connstate_t;
pub use ct_table::ct_table_t;
pub use cvar::{cvarHandle_t, cvar_s, cvar_t, vmCvar_t, MAX_CVAR_VALUE_STRING};
pub use effect_trail_arg::effectTrailArgStruct_t;
pub use effect_trail_vert::effectTrailVertStruct_t;
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
pub use game_state::{gameState_t, MAX_CONFIGSTRINGS, MAX_GAMESTATE_CHARS};
pub use gen_cmds::genCmds_t;
pub use hunk_pref::ha_pref;
pub use int64::qint64;
pub use item_use_fail::itemUseFail_t;
pub use limits::{
    BIG_INFO_STRING, ENTITYNUM_MAX_NORMAL, ENTITYNUM_NONE, ENTITYNUM_WORLD, GENTITYNUM_BITS,
    MAX_CLIENTS, MAX_CLIENTS_I32, MAX_GENTITIES, MAX_INFO_STRING, MAX_STRING_CHARS,
    SNAPFLAG_SERVERCOUNT,
};
pub use mark_fragment::markFragment_t;
pub use pc_token::{pc_token_t, MAX_TOKENLENGTH};
pub use print_parm::printParm_t;
pub use q_color::{Q_IsColorString, Q_COLOR_ESCAPE};
pub use q_math::{_DotProduct, _VectorCopy, _VectorSubtract, VectorNormalize};
pub use q_math_rand::RAND_MAX;
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
pub use surface_flags::{
    CONTENTS_ABSEIL, CONTENTS_BODY, CONTENTS_BOTCLIP, CONTENTS_CORPSE, CONTENTS_DETAIL,
    CONTENTS_FOG, CONTENTS_INSIDE, CONTENTS_ITEM, CONTENTS_LADDER, CONTENTS_LAVA,
    CONTENTS_LIGHTSABER, CONTENTS_MONSTERCLIP, CONTENTS_NODROP, CONTENTS_NOSHOT, CONTENTS_OPAQUE,
    CONTENTS_OUTSIDE, CONTENTS_PLAYERCLIP, CONTENTS_SHOTCLIP, CONTENTS_SLIME, CONTENTS_SOLID,
    CONTENTS_TELEPORTER, CONTENTS_TERRAIN, CONTENTS_TRANSLUCENT, CONTENTS_TRIGGER, CONTENTS_WATER,
    MASK_ALL, MASK_DEADSOLID, MASK_NPCSOLID, MASK_OPAQUE, MASK_PLAYERSOLID, MASK_SHOT, MASK_SOLID,
    MASK_WATER, SURF_FORCEFIELD, SURF_METALSTEPS, SURF_NODAMAGE, SURF_NODLIGHT, SURF_NODRAW,
    SURF_NOIMPACT, SURF_NOMARKS, SURF_NOMISCENTS, SURF_NOSTEPS, SURF_SKY, SURF_SLICK,
};
pub use trackchan::trackchan_t;
pub use trajectory::{trType_t, trajectory_t};
pub use vec3struct::vec3struct_t;
pub use vector::{vec2_t, vec3_t, vec3pair_t, vec4_t, vec5_t, vec_t};
pub use wl_e::WL_e;
pub use world_size::WORLD_SIZE;
pub use wpneighbor::wpneighbor_t;
pub use wpobject::{wpobject_t, MAX_NEIGHBOR_SIZE};

// Migration bridge: cross-mode scalar/handle primitives now live in `native_types`.
pub use native_types::{
    clipHandle_t, fileHandle_t, fxHandle_t, mdxaBone_t, qboolean, qfalse, qhandle_t, qtrue,
    sfxHandle_t, MAX_QPATH,
};

pub use native_math::eorientations::Eorientations;
