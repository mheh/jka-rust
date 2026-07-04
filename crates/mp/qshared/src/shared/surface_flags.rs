//! MP `surfaceflags.h` `CONTENTS_*`/`SURF_*` bit values and their
//! `bg_public.h` masks.
//!
//! Plain `#define` bit flags (not an enum), so §C8 makes them `const`s
//! directly.
//!
//! Source: `oracle/oracle/codemp/game/surfaceflags.h:10-45`

use core::ffi::c_int;

pub const CONTENTS_SOLID: c_int = 0x0000_0001; // Default setting. An eye is never valid in a solid.
pub const CONTENTS_LAVA: c_int = 0x0000_0002;
pub const CONTENTS_WATER: c_int = 0x0000_0004;
pub const CONTENTS_FOG: c_int = 0x0000_0008;
pub const CONTENTS_PLAYERCLIP: c_int = 0x0000_0010;
pub const CONTENTS_MONSTERCLIP: c_int = 0x0000_0020; // Physically block bots
pub const CONTENTS_BOTCLIP: c_int = 0x0000_0040; // A hint for bots - do not enter this brush by navigation (if possible)
pub const CONTENTS_SHOTCLIP: c_int = 0x0000_0080;
pub const CONTENTS_BODY: c_int = 0x0000_0100; // should never be on a brush, only in game
pub const CONTENTS_CORPSE: c_int = 0x0000_0200; // should never be on a brush, only in game
pub const CONTENTS_TRIGGER: c_int = 0x0000_0400;
pub const CONTENTS_NODROP: c_int = 0x0000_0800; // don't leave bodies or items (death fog, lava)
pub const CONTENTS_TERRAIN: c_int = 0x0000_1000; // volume contains terrain data
pub const CONTENTS_LADDER: c_int = 0x0000_2000;
pub const CONTENTS_ABSEIL: c_int = 0x0000_4000; // (SOF2) used like ladder to define where an NPC can abseil
pub const CONTENTS_OPAQUE: c_int = 0x0000_8000; // defaults to on, when off, solid can be seen through
pub const CONTENTS_OUTSIDE: c_int = 0x0001_0000; // volume is considered to be in the outside (i.e. not indoors)
pub const CONTENTS_SLIME: c_int = 0x0002_0000; // CHC needs this since we use same tools
pub const CONTENTS_LIGHTSABER: c_int = 0x0004_0000; // ""
pub const CONTENTS_TELEPORTER: c_int = 0x0008_0000; // ""
pub const CONTENTS_ITEM: c_int = 0x0010_0000; // ""
pub const CONTENTS_NOSHOT: c_int = 0x0020_0000; // shots pass through me
pub const CONTENTS_DETAIL: c_int = 0x0800_0000; // brushes not used for the bsp
pub const CONTENTS_INSIDE: c_int = 0x1000_0000; // volume is considered to be inside (i.e. indoors)
pub const CONTENTS_TRANSLUCENT: c_int = 0x8000_0000u32 as c_int; // don't consume surface fragments inside

pub const SURF_SKY: c_int = 0x0000_2000; // lighting from environment map
pub const SURF_SLICK: c_int = 0x0000_4000; // affects game physics
pub const SURF_METALSTEPS: c_int = 0x0000_8000; // CHC needs this since we use same tools (though this flag is temp?)
pub const SURF_FORCEFIELD: c_int = 0x0001_0000; // CHC "" (but not temp)
pub const SURF_NODAMAGE: c_int = 0x0004_0000; // never give falling damage
pub const SURF_NOIMPACT: c_int = 0x0008_0000; // don't make missile explosions
pub const SURF_NOMARKS: c_int = 0x0010_0000; // don't leave missile marks
pub const SURF_NODRAW: c_int = 0x0020_0000; // don't generate a drawsurface at all
pub const SURF_NOSTEPS: c_int = 0x0040_0000; // no footstep sounds
pub const SURF_NODLIGHT: c_int = 0x0080_0000; // don't dlight even if solid (solid lava, skies)
pub const SURF_NOMISCENTS: c_int = 0x0100_0000; // no client models allowed on this surface

/// Raven `MASK_ALL`.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:1170`
pub const MASK_ALL: c_int = -1;

/// Raven `MASK_SOLID`.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:1171`
pub const MASK_SOLID: c_int = CONTENTS_SOLID | CONTENTS_TERRAIN;

/// Raven `MASK_PLAYERSOLID`.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:1172`
pub const MASK_PLAYERSOLID: c_int = CONTENTS_SOLID | CONTENTS_PLAYERCLIP | CONTENTS_BODY | CONTENTS_TERRAIN;

/// Raven `MASK_NPCSOLID`.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:1173`
pub const MASK_NPCSOLID: c_int =
    CONTENTS_SOLID | CONTENTS_MONSTERCLIP | CONTENTS_BODY | CONTENTS_TERRAIN;

/// Raven `MASK_DEADSOLID`.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:1174`
pub const MASK_DEADSOLID: c_int = CONTENTS_SOLID | CONTENTS_PLAYERCLIP | CONTENTS_TERRAIN;

/// Raven `MASK_WATER`.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:1175`
pub const MASK_WATER: c_int = CONTENTS_WATER | CONTENTS_LAVA | CONTENTS_SLIME;

/// Raven `MASK_OPAQUE`.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:1176`
pub const MASK_OPAQUE: c_int = CONTENTS_SOLID | CONTENTS_SLIME | CONTENTS_LAVA | CONTENTS_TERRAIN;

/// Raven `MASK_SHOT`.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:1177`
pub const MASK_SHOT: c_int = CONTENTS_SOLID | CONTENTS_BODY | CONTENTS_CORPSE | CONTENTS_TERRAIN;
