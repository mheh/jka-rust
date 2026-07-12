//! MP `playerState_t::pm_flags` bit values (`pmove->pm_flags`).
//!
//! Plain `#define` bit flags (not an enum), so §C8 makes them `const`s
//! directly.
//!
//! Source: `oracle/codemp/game/bg_public.h:403-417`

use core::ffi::c_int;

pub const PMF_DUCKED: c_int = 1;
pub const PMF_JUMP_HELD: c_int = 2;
pub const PMF_ROLLING: c_int = 4;
pub const PMF_BACKWARDS_JUMP: c_int = 8; // go into backwards land
pub const PMF_BACKWARDS_RUN: c_int = 16; // coast down to backwards run
pub const PMF_TIME_LAND: c_int = 32; // pm_time is time before rejump
pub const PMF_TIME_KNOCKBACK: c_int = 64; // pm_time is an air-accelerate only time
pub const PMF_FIX_MINS: c_int = 128; // mins have been brought up, keep tracing down to fix them
pub const PMF_TIME_WATERJUMP: c_int = 256; // pm_time is waterjump
pub const PMF_RESPAWNED: c_int = 512; // clear after attack and jump buttons come up
pub const PMF_USE_ITEM_HELD: c_int = 1024;
pub const PMF_UPDATE_ANIM: c_int = 2048; // The server updated the animation, the pmove should set the ghoul2 anim to match.
pub const PMF_FOLLOW: c_int = 4096; // spectate following another player
pub const PMF_SCOREBOARD: c_int = 8192; // spectate as a scoreboard
pub const PMF_STUCK_TO_WALL: c_int = 16384; // grabbing a wall

/// Raven `PMF_ALL_TIMES`.
///
/// Source: `oracle/codemp/game/bg_public.h:419`
pub const PMF_ALL_TIMES: c_int = PMF_TIME_WATERJUMP | PMF_TIME_LAND | PMF_TIME_KNOCKBACK;
