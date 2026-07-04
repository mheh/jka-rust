//! MP `surfaceflags.h` `CONTENTS_*` bit values and their `bg_public.h` masks.
//!
//! Plain `#define` bit flags (not an enum), so §C8 makes them `const`s
//! directly. Only the subset the mega-pass logic port actually references is
//! transcribed here (§E13, slice-driven).
//!
//! Source: `oracle/oracle/codemp/game/surfaceflags.h:10-22`

use core::ffi::c_int;

pub const CONTENTS_SOLID: c_int = 0x0000_0001; // Default setting. An eye is never valid in a solid.
pub const CONTENTS_BODY: c_int = 0x0000_0100; // should never be on a brush, only in game
pub const CONTENTS_CORPSE: c_int = 0x0000_0200; // should never be on a brush, only in game
pub const CONTENTS_TERRAIN: c_int = 0x0000_1000; // volume contains terrain data

/// Raven `MASK_SOLID`.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:1171`
pub const MASK_SOLID: c_int = CONTENTS_SOLID | CONTENTS_TERRAIN;
