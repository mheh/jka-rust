//! MP `surfaceflags.h` `CONTENTS_*`/`SURF_*` bit values and their
//! `bg_public.h` masks.
//!
//! Plain `#define` bit flags (not an enum), so §C8 makes them `const`s
//! directly.
//!
//! Source: `oracle/codemp/game/surfaceflags.h:10-45`

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

// --- `surfaceflags.h` `MATERIAL_*` ground-material bits (surface type, packed
// into `groundTrace.surfaceFlags` and masked out via `MATERIAL_MASK`).
// Source: `oracle/codemp/game/surfaceflags.h:51-86`
pub const MATERIAL_BITS: c_int = 5;
pub const MATERIAL_MASK: c_int = 0x1f;

pub const MATERIAL_NONE: c_int = 0; // for when the artist hasn't set anything up =)
pub const MATERIAL_SOLIDWOOD: c_int = 1; // freshly cut timber
pub const MATERIAL_HOLLOWWOOD: c_int = 2; // termite infested creaky wood
pub const MATERIAL_SOLIDMETAL: c_int = 3; // solid girders
pub const MATERIAL_HOLLOWMETAL: c_int = 4; // hollow metal machines
pub const MATERIAL_SHORTGRASS: c_int = 5; // manicured lawn
pub const MATERIAL_LONGGRASS: c_int = 6; // long jungle grass
pub const MATERIAL_DIRT: c_int = 7; // hard mud
pub const MATERIAL_SAND: c_int = 8; // sandy beach
pub const MATERIAL_GRAVEL: c_int = 9; // lots of small stones
pub const MATERIAL_GLASS: c_int = 10;
pub const MATERIAL_CONCRETE: c_int = 11; // hardened concrete pavement
pub const MATERIAL_MARBLE: c_int = 12; // marble floors
pub const MATERIAL_WATER: c_int = 13; // light covering of water on a surface
pub const MATERIAL_SNOW: c_int = 14; // freshly laid snow
pub const MATERIAL_ICE: c_int = 15; // packed snow/solid ice
pub const MATERIAL_FLESH: c_int = 16; // hung meat, corpses in the world
pub const MATERIAL_MUD: c_int = 17; // wet soil
pub const MATERIAL_BPGLASS: c_int = 18; // bulletproof glass
pub const MATERIAL_DRYLEAVES: c_int = 19; // dried up leaves on the floor
pub const MATERIAL_GREENLEAVES: c_int = 20; // fresh leaves still on a tree
pub const MATERIAL_FABRIC: c_int = 21; // Cotton sheets
pub const MATERIAL_CANVAS: c_int = 22; // tent material
pub const MATERIAL_ROCK: c_int = 23;
pub const MATERIAL_RUBBER: c_int = 24; // hard tire like rubber
pub const MATERIAL_PLASTIC: c_int = 25;
pub const MATERIAL_TILES: c_int = 26; // tiled floor
pub const MATERIAL_CARPET: c_int = 27; // lush carpet
pub const MATERIAL_PLASTER: c_int = 28; // drywall style plaster
pub const MATERIAL_SHATTERGLASS: c_int = 29; // glass with the Crisis Zone style shattering
pub const MATERIAL_ARMOR: c_int = 30; // body armor
pub const MATERIAL_COMPUTER: c_int = 31; // computers/electronic equipment
pub const MATERIAL_LAST: c_int = 32; // number of materials

/// Raven `MATERIALS` — parallel name table for the `MATERIAL_*` constants
/// above, index-for-index (`"none"` = [`MATERIAL_NONE`], `"solidwood"` =
/// [`MATERIAL_SOLIDWOOD`], ...). Defined as an X-macro in Raven "so one
/// change will affect all the relevant files".
///
/// Source: `oracle/codemp/game/surfaceflags.h:90-123`
pub const MATERIALS: [&str; 32] = [
    "none",
    "solidwood",
    "hollowwood",
    "solidmetal",
    "hollowmetal",
    "shortgrass",
    "longgrass",
    "dirt",
    "sand",
    "gravel",
    "glass",
    "concrete",
    "marble",
    "water",
    "snow",
    "ice",
    "flesh",
    "mud",
    "bpglass",
    "dryleaves",
    "greenleaves",
    "fabric",
    "canvas",
    "rock",
    "rubber",
    "plastic",
    "tiles",
    "carpet",
    "plaster",
    "shatterglass",
    "armor",
    "computer",
];

/// Raven `MASK_ALL`.
///
/// Source: `oracle/codemp/game/bg_public.h:1170`
pub const MASK_ALL: c_int = -1;

/// Raven `MASK_SOLID`.
///
/// Source: `oracle/codemp/game/bg_public.h:1171`
pub const MASK_SOLID: c_int = CONTENTS_SOLID | CONTENTS_TERRAIN;

/// Raven `MASK_PLAYERSOLID`.
///
/// Source: `oracle/codemp/game/bg_public.h:1172`
pub const MASK_PLAYERSOLID: c_int = CONTENTS_SOLID | CONTENTS_PLAYERCLIP | CONTENTS_BODY | CONTENTS_TERRAIN;

/// Raven `MASK_NPCSOLID`.
///
/// Source: `oracle/codemp/game/bg_public.h:1173`
pub const MASK_NPCSOLID: c_int =
    CONTENTS_SOLID | CONTENTS_MONSTERCLIP | CONTENTS_BODY | CONTENTS_TERRAIN;

/// Raven `MASK_DEADSOLID`.
///
/// Source: `oracle/codemp/game/bg_public.h:1174`
pub const MASK_DEADSOLID: c_int = CONTENTS_SOLID | CONTENTS_PLAYERCLIP | CONTENTS_TERRAIN;

/// Raven `MASK_WATER`.
///
/// Source: `oracle/codemp/game/bg_public.h:1175`
pub const MASK_WATER: c_int = CONTENTS_WATER | CONTENTS_LAVA | CONTENTS_SLIME;

/// Raven `MASK_OPAQUE`.
///
/// Source: `oracle/codemp/game/bg_public.h:1176`
pub const MASK_OPAQUE: c_int = CONTENTS_SOLID | CONTENTS_SLIME | CONTENTS_LAVA | CONTENTS_TERRAIN;

/// Raven `MASK_SHOT`.
///
/// Source: `oracle/codemp/game/bg_public.h:1177`
pub const MASK_SHOT: c_int = CONTENTS_SOLID | CONTENTS_BODY | CONTENTS_CORPSE | CONTENTS_TERRAIN;

/// Raven `SOLID_BMODEL` — an entity's `solid` field is set to this sentinel
/// when the entity uses its brush model as its collision shape.
///
/// Source: `oracle/codemp/game/q_shared.h:2642`
pub const SOLID_BMODEL: c_int = 0xffffff;
