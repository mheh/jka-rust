//! MP `bg_public.h` `entityState_t::eFlags`/`eFlags2` bit values.
//!
//! Plain `#define` bit flags (not an enum), so §C8 makes them `const`s
//! directly. Only the subset the mega-pass logic port actually references is
//! transcribed here (§E13, slice-driven).
//!
//! Source: `oracle/oracle/codemp/game/bg_public.h:558-621`

use core::ffi::c_int;

pub const EF_DEAD: c_int = 1 << 1; // don't draw a foe marker over players with EF_DEAD
pub const EF_NODRAW: c_int = 1 << 8; // may have an event, but no model (unspawned items)
pub const EF_DROPPEDWEAPON: c_int = 1 << 25; // it's a dropped weapon
pub const EF_INVULNERABLE: c_int = 1 << 27; // just spawned in or whatever, so is protected
pub const EF_SEEKERDRONE: c_int = 1 << 21; // show seeker drone floating around head

pub const EF2_HELD_BY_MONSTER: c_int = 1 << 0; // Being held by something, like a Rancor or a Wampa
pub const EF2_FLYING: c_int = 1 << 4; // Flying (NPC-only)
