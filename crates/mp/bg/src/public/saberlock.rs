//! MP `bg_public.h` saber-lock stage constants.
//!
//! Source: `oracle/codemp/game/bg_public.h:208-217`

#![allow(non_camel_case_types, non_upper_case_globals)]

use core::ffi::c_int;

// Raven declares these in a `typedef enum { ... };` with no type name (the
// `typedef` binds no identifier), so callers (`G_SaberLockAnim` in
// `w_saber.c:1094`) take them as plain `int` — ported as loose consts, not an
// enum, matching porting-rules' "anonymous enum -> consts" rule.
// Source: `oracle/codemp/game/bg_public.h:208-217`
pub const SABERLOCK_TOP: c_int = 0;
pub const SABERLOCK_SIDE: c_int = 1;
pub const SABERLOCK_LOCK: c_int = 2;
pub const SABERLOCK_BREAK: c_int = 3;
pub const SABERLOCK_SUPERBREAK: c_int = 4;
pub const SABERLOCK_WIN: c_int = 5;
pub const SABERLOCK_LOSE: c_int = 6;

/// `sabersLockMode_t` (MP).
///
/// MP variant differs from SP's file: only LOCK_FIRST..LOCK_RANDOM, no
/// LOCK_KYLE_GRAB*/LOCK_FORCE_DRAIN.
/// Source: `oracle/codemp/game/w_saber.c:1077-1086`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum sabersLockMode_t {
	LOCK_FIRST = 0,
	LOCK_TOP,
	LOCK_DIAG_TR,
	LOCK_DIAG_TL,
	LOCK_DIAG_BR,
	LOCK_DIAG_BL,
	LOCK_R,
	LOCK_L,
	LOCK_RANDOM,
}
pub use sabersLockMode_t::*;

/// Ideal saber-lock distances (Raven `#define`s).
/// Source: `oracle/codemp/game/w_saber.c:1088-1089`
pub const LOCK_IDEAL_DIST_TOP: f32 = 32.0;
pub const LOCK_IDEAL_DIST_CIRCLE: f32 = 48.0;
// Richard Lico wanted this value tweaked for the JKA lock distance.
// Source: `oracle/codemp/game/w_saber.c:1216`
pub const LOCK_IDEAL_DIST_JKA: f32 = 46.0;
