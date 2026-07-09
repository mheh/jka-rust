//! MP `bg_public.h` `NPC_SetAnim`/`BG_SetAnim` body-part selectors and flags.
//!
//! Plain `#define`s (not an enum), so §C8 makes them `const`s directly.
//!
//! Source: `oracle/codemp/game/bg_public.h:498-506`

use core::ffi::c_int;

pub const SETANIM_TORSO: c_int = 1;
pub const SETANIM_LEGS: c_int = 2;
pub const SETANIM_BOTH: c_int = SETANIM_TORSO | SETANIM_LEGS;

pub const SETANIM_FLAG_NORMAL: c_int = 0; // Only set if timer is 0
pub const SETANIM_FLAG_OVERRIDE: c_int = 1; // Override previous
pub const SETANIM_FLAG_HOLD: c_int = 2; // Set the new timer
pub const SETANIM_FLAG_RESTART: c_int = 4; // Allow restarting the anim if playing the same one (weapon fires)
pub const SETANIM_FLAG_HOLDLESS: c_int = 8; // Set the new timer
