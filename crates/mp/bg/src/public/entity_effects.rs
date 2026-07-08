//! MP `bg_public.h` `entityState_t::eFlags2` (`EF2_*`) NPC flag bits.
//!
//! Raven defines `EF_*` (`eFlags`) and `EF2_*` (`eFlags2`) once in
//! `bg_public.h`. The `EF_*` set is owned by [`super::entity_flags`]; this
//! module owns the distinct `EF2_*` set and re-exports `EF_*` so existing
//! `entity_effects::EF_*` import paths keep resolving.
//!
//! Plain `#define` bit flags (not an enum), so §C8 makes them `const`s
//! directly.
//!
//! Source: `oracle/oracle/codemp/game/bg_public.h:558-624`

use core::ffi::c_int;

// Re-export the `EF_*` (`eFlags`) bits from their canonical home so callers of
// `entity_effects::EF_*` (and `prelude::*`) still resolve them.
pub use super::entity_flags::*;

// These new EF2_??? flags were added for NPCs, they really should not be used
// often. NOTE: we only allow 10 of these!
pub const EF2_HELD_BY_MONSTER: c_int = 1 << 0; // Being held by something, like a Rancor or a Wampa
pub const EF2_USE_ALT_ANIM: c_int = 1 << 1; // For certain special runs/stands for creatures like the Rancor and Wampa whose runs/stands are conditional
pub const EF2_ALERTED: c_int = 1 << 2; // For certain special anims, for Rancor: means you've had an enemy, so use the more alert stand
pub const EF2_GENERIC_NPC_FLAG: c_int = 1 << 3; // So far, used for Rancor...
pub const EF2_FLYING: c_int = 1 << 4; // Flying FIXME: only used on NPCs doesn't *really* have to be passed over, does it?
pub const EF2_HYPERSPACE: c_int = 1 << 5; // Used to both start the hyperspace effect on the predicted client and to let the vehicle know it can now jump into hyperspace (after turning to face the proper angle)
pub const EF2_BRACKET_ENTITY: c_int = 1 << 6; // Draw as bracketed
pub const EF2_SHIP_DEATH: c_int = 1 << 7; // "died in ship" mode
pub const EF2_NOT_USED_1: c_int = 1 << 8; // not used
