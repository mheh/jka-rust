//! Globally-shared wire & math types (Raven `q_shared.h` scope): vec3, entityState, playerState, trace, usercmd.

#![allow(non_camel_case_types)]

pub mod cbuf_exec;
pub mod collision;
pub mod connstate;
pub mod ct_table;
pub mod cvar;
#[path = "e_status.rs"]
pub mod cinematic_status;
pub mod entity_shared;
pub mod error_parm;
pub mod force_powers;
pub mod force_reload;
pub mod fs_origin;
pub mod game_state;
pub mod gen_cmds;
pub mod parse_data;
pub mod shared_ragdoll_update_params;
#[path = "ivec2_t.rs"]
pub mod ivec2;
pub mod limits;
pub mod lpcstr;
pub mod mark_fragment;
pub mod print_parm;
pub mod shared_eik_move_state;
pub mod string_id_table;
pub mod water_height_level;
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

pub use cbuf_exec::cbufExec_t;
pub use cinematic_status::e_status;
pub use collision::{cplane_t, CollisionRecord_t};
pub use connstate::connstate_t;
pub use ct_table::ct_table_t;
pub use cvar::{cvarHandle_t, vmCvar_t, MAX_CVAR_VALUE_STRING};
pub use entity_shared::entityShared_t;
pub use error_parm::errorParm_t;
pub use file_mode::{fsMode_t, FS_APPEND, FS_APPEND_SYNC, FS_READ, FS_WRITE};
pub use force_powers::{
    forcePowers_t, FP_ABSORB, FP_DRAIN, FP_FIRST, FP_GRIP, FP_HEAL, FP_LEVITATION, FP_LIGHTNING,
    FP_PROTECT, FP_PULL, FP_PUSH, FP_RAGE, FP_SABERTHROW, FP_SABER_DEFENSE, FP_SABER_OFFENSE,
    FP_SEE, FP_SPEED, FP_TELEPATHY, NUM_FORCE_POWERS,
};
pub use force_reload::ForceReload_e;
pub use fs_origin::fsOrigin_t;
pub use game_state::{gameState_t, MAX_CONFIGSTRINGS, MAX_GAMESTATE_CHARS};
pub use gen_cmds::genCmds_t;
pub use parse_data::{parseData_t, MAX_PARSEFILES};
pub use shared_ragdoll_update_params::sharedRagDollUpdateParams_t;
pub use ivec2::ivec2_t;
pub use limits::{MAX_CLIENTS, MAX_STRING_CHARS};
pub use lpcstr::LPCSTR;
pub use mark_fragment::markFragment_t;
pub use pc_token::{pc_token_t, MAX_TOKENLENGTH};
pub use print_parm::printParm_t;
pub use shared_eik_move_state::sharedEIKMoveState;
pub use shared_ik_move_params::sharedIKMoveParams_t;
pub use string_id_table::stringID_table_t;
pub use trajectory::{trType_t, trajectory_t};
pub use water_height_level::waterHeightLevel_t;
pub use vector::{vec2_t, vec3_t, vec3pair_t, vec4_t, vec5_t, vec_t};

// Migration bridge: cross-mode scalar/handle primitives now live in `native_types`.
pub use native_types::{
    clipHandle_t, fileHandle_t, fxHandle_t, mdxaBone_t, qboolean, qhandle_t, sfxHandle_t,
    MAX_QPATH, QFALSE, QTRUE,
};
// Cross-mode math types from `native_math` needed by the SP abi fn tables.
pub use native_math::eorientations::Eorientations;
pub use native_math::orientation::orientation_t;
