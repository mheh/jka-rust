//! MP `bg_pmove.c` per-force-level scalar tuning tables.

#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use crate::public::JUMP_VELOCITY;

/// Raven `forceSpeedLevels[4]` — FP_SPEED move-speed multiplier per force
/// level.
///
/// Extern decl source: `oracle/codemp/game/q_shared.h:435`
/// Definition source: `oracle/codemp/game/bg_pmove.c:59-65`
pub static forceSpeedLevels: [f32; 4] = [
    1.0, //rank 0?
    1.25,
    1.5,
    1.75,
];

/// Raven `forceJumpHeight[NUM_FORCE_POWER_LEVELS]` — force-jump max height
/// per level.
///
/// Extern decl source: `oracle/codemp/game/w_saber.h:71`
/// Definition source: `oracle/codemp/game/bg_pmove.c:155-161`
pub static forceJumpHeight: [f32; 4] = [
    32.0,  //normal jump (+stepheight+crouchdiff = 66)
    96.0,  //(+stepheight+crouchdiff = 130)
    192.0, //(+stepheight+crouchdiff = 226)
    384.0, //(+stepheight+crouchdiff = 418)
];

/// Raven `forceJumpStrength[NUM_FORCE_POWER_LEVELS]` — force-jump vertical
/// launch velocity per level; element 0 is the normal-jump `JUMP_VELOCITY`.
///
/// Extern decl source: `oracle/codemp/game/w_saber.h:72`
/// Definition source: `oracle/codemp/game/bg_pmove.c:163-169`
pub static forceJumpStrength: [f32; 4] = [
    JUMP_VELOCITY, //normal jump
    420.0,
    590.0,
    840.0,
];

/// Raven `forceJumpHeightMax[NUM_FORCE_POWER_LEVELS]` — force-jump max height
/// per level including stepheight/crouchdiff (used by `PM_CheckJump`'s
/// max-height clamp, distinct from `forceJumpHeight`).
///
/// Definition source: `oracle/codemp/game/bg_pmove.c:1768-1774`
pub static forceJumpHeightMax: [f32; 4] = [
    66.0,  //normal jump (32+stepheight(18)+crouchdiff(24) = 74)
    130.0, //(96+stepheight(18)+crouchdiff(24) = 138)
    226.0, //(192+stepheight(18)+crouchdiff(24) = 234)
    418.0, //(384+stepheight(18)+crouchdiff(24) = 426)
];
