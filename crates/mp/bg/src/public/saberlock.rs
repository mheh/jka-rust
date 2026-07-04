//! MP `bg_public.h` saber-lock stage constants.
//!
//! Source: `oracle/oracle/codemp/game/bg_public.h:208-217`

#![allow(non_camel_case_types, non_upper_case_globals)]

use core::ffi::c_int;

// Raven declares these in a `typedef enum { ... };` with no type name (the
// `typedef` binds no identifier), so callers (`G_SaberLockAnim` in
// `w_saber.c:1094`) take them as plain `int` — ported as loose consts, not an
// enum, matching porting-rules' "anonymous enum -> consts" rule.
// Source: `oracle/oracle/codemp/game/bg_public.h:208-217`
pub const SABERLOCK_TOP: c_int = 0;
pub const SABERLOCK_SIDE: c_int = 1;
pub const SABERLOCK_LOCK: c_int = 2;
pub const SABERLOCK_BREAK: c_int = 3;
pub const SABERLOCK_SUPERBREAK: c_int = 4;
pub const SABERLOCK_WIN: c_int = 5;
pub const SABERLOCK_LOSE: c_int = 6;
