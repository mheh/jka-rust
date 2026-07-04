//! MP `bg_public.h` `g_dmflags` cvar integer bit values.
//!
//! Plain `#define` bit flags (not an enum), so §C8 makes them `const`s
//! directly.
//!
//! Source: `oracle/oracle/codemp/game/bg_public.h:1163-1165`

use core::ffi::c_int;

pub const DF_NO_FALLING: c_int = 8;
pub const DF_FIXED_FOV: c_int = 16;
pub const DF_NO_FOOTSTEPS: c_int = 32;
