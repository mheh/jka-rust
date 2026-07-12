//! MP `bg_public.h` `ET_FX` states (stored in `entityState_t::modelindex2`).
//!
//! Plain `#define`s (not an enum), so §C8 makes them `const`s directly.
//!
//! Source: `oracle/codemp/game/bg_public.h:1177-1185`

use core::ffi::c_int;

pub const FX_STATE_OFF: c_int = 0;
pub const FX_STATE_ONE_SHOT: c_int = 1;
pub const FX_STATE_ONE_SHOT_LIMIT: c_int = 10;
pub const FX_STATE_CONTINUOUS: c_int = 20;
