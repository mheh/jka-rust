//! MP `bg_misc.c` toggleable-surface name/debris tables.

#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use core::ffi::{c_int, CStr};

/// Raven `bgToggleableSurfaces[BG_NUM_TOGGLEABLE_SURFACES]` — Ghoul2 surface
/// names that can be toggled on/off (vehicle parts, cyborg canisters, etc.);
/// scanned by consumers until the `NULL` sentinel (`None`) is hit.
///
/// Definition source: `oracle/codemp/game/bg_misc.c:34-76`
/// Extern decl source: `oracle/codemp/game/bg_public.h:145`
pub static bgToggleableSurfaces: [Option<&CStr>; 31] = [
    Some(c"l_arm_key"), //0
    Some(c"torso_canister1"),
    Some(c"torso_canister2"),
    Some(c"torso_canister3"),
    Some(c"torso_tube1"),
    Some(c"torso_tube2"), //5
    Some(c"torso_tube3"),
    Some(c"torso_tube4"),
    Some(c"torso_tube5"),
    Some(c"torso_tube6"),
    Some(c"r_arm"), //10
    Some(c"l_arm"),
    Some(c"torso_shield"),
    Some(c"torso_galaktorso"),
    Some(c"torso_collar"),
    // "torso_eyes_mouth",              //15
    // "torso_galakhead",
    // "torso_galakface",
    // "torso_antenna_base_cap",
    // "torso_antenna",
    // "l_arm_augment",                //20
    // "l_arm_middle",
    // "l_arm_wrist",
    // "r_arm_middle", //yeah.. galak's surf stuff is no longer auto, sorry! need the space for vehicle surfs.
    Some(c"r_wing1"), //15
    Some(c"r_wing2"),
    Some(c"l_wing1"),
    Some(c"l_wing2"),
    Some(c"r_gear"),
    Some(c"l_gear"), //20
    Some(c"nose"),
    Some(c"blah4"),
    Some(c"blah5"),
    Some(c"l_hand"),
    Some(c"r_hand"), //25
    Some(c"helmet"),
    Some(c"head"),
    Some(c"head_concussion_charger"),
    Some(c"head_light_blaster_cann"), //29
    None,
];

/// Raven `bgToggleableSurfaceDebris[BG_NUM_TOGGLEABLE_SURFACES]` — debris
/// effect index paired 1:1 with `bgToggleableSurfaces`; `-1` marks the
/// sentinel slot past the real entries.
///
/// Definition source: `oracle/codemp/game/bg_misc.c:78-111`
/// Extern decl source: `oracle/codemp/game/bg_public.h:146`
pub static bgToggleableSurfaceDebris: [c_int; 31] = [
    0, //0
    0,
    0,
    0,
    0,
    0, //5
    0,
    0,
    0,
    0,
    0, //10
    0,
    0,
    0,
    0, //>= 2 means it should create a flame trail when destroyed (for vehicles)
    3, //15
    5, //rwing2
    4,
    6, //lwing2
    0, //rgear
    0, //lgear //20
    7, //nose
    0, //blah
    0, //blah
    0,
    0, //25
    0,
    0,
    0,
    0, //29
    -1,
];
