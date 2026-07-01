//! Globally-shared wire & math types (Raven `q_shared.h` scope): vec3, entityState, playerState, trace, usercmd.

#![allow(non_camel_case_types)]

pub mod collision;
pub mod cvar;
pub mod entity_shared;
pub mod limits;
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

pub use collision::{cplane_t, CollisionRecord_t};
pub use cvar::{cvarHandle_t, vmCvar_t, MAX_CVAR_VALUE_STRING};
pub use entity_shared::entityShared_t;
pub use file_mode::{fsMode_t, FS_APPEND, FS_APPEND_SYNC, FS_READ, FS_WRITE};
pub use limits::{MAX_CLIENTS, MAX_STRING_CHARS};
pub use pc_token::{pc_token_t, MAX_TOKENLENGTH};
pub use shared_ik_move_params::sharedIKMoveParams_t;
pub use trajectory::{trType_t, trajectory_t};
pub use vector::{vec2_t, vec3_t, vec3pair_t, vec4_t, vec5_t, vec_t};

// Migration bridge: cross-mode scalar/handle primitives now live in `native_types`.
pub use native_types::{
    clipHandle_t, fileHandle_t, mdxaBone_t, qboolean, qhandle_t, MAX_QPATH, QFALSE, QTRUE,
};
