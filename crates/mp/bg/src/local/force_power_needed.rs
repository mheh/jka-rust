//! MP `bg_pmove.c` force-power point-cost table.

#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use core::ffi::c_int;

/// Raven `forcePowerNeeded[NUM_FORCE_POWER_LEVELS][NUM_FORCE_POWERS]` — force
/// points required to activate/maintain each power at each of the 4 force
/// levels. Column order matches `forcePowers_t`: FP_HEAL, FP_LEVITATION,
/// FP_SPEED, FP_PUSH, FP_PULL, FP_TELEPATHY, FP_GRIP, FP_LIGHTNING, FP_RAGE,
/// FP_PROTECT, FP_ABSORB, FP_TEAM_HEAL, FP_TEAM_FORCE, FP_DRAIN, FP_SEE,
/// FP_SABER_OFFENSE, FP_SABER_DEFENSE, FP_SABERTHROW.
///
/// Definition source: `oracle/codemp/game/bg_pmove.c:67-153`
/// Extern decl source: `oracle/codemp/game/bg_local.h:54`
pub static forcePowerNeeded: [[c_int; 18]; 4] = [
    [
        999, //FP_HEAL,//instant
        999, //FP_LEVITATION,//hold/duration
        999, //FP_SPEED,//duration
        999, //FP_PUSH,//hold/duration
        999, //FP_PULL,//hold/duration
        999, //FP_TELEPATHY,//instant
        999, //FP_GRIP,//hold/duration
        999, //FP_LIGHTNING,//hold/duration
        999, //FP_RAGE,//duration
        999, //FP_PROTECT,//duration
        999, //FP_ABSORB,//duration
        999, //FP_TEAM_HEAL,//instant
        999, //FP_TEAM_FORCE,//instant
        999, //FP_DRAIN,//hold/duration
        999, //FP_SEE,//duration
        999, //FP_SABER_OFFENSE,
        999, //FP_SABER_DEFENSE,
        999, //FP_SABERTHROW,
    ],
    [
        65, //FP_HEAL,//instant //was 25, but that was way too little
        10, //FP_LEVITATION,//hold/duration
        50, //FP_SPEED,//duration
        20, //FP_PUSH,//hold/duration
        20, //FP_PULL,//hold/duration
        20, //FP_TELEPATHY,//instant
        30, //FP_GRIP,//hold/duration
        1,  //FP_LIGHTNING,//hold/duration
        50, //FP_RAGE,//duration
        50, //FP_PROTECT,//duration
        50, //FP_ABSORB,//duration
        50, //FP_TEAM_HEAL,//instant
        50, //FP_TEAM_FORCE,//instant
        20, //FP_DRAIN,//hold/duration
        20, //FP_SEE,//duration
        0,  //FP_SABER_OFFENSE,
        2,  //FP_SABER_DEFENSE,
        20, //FP_SABERTHROW,
    ],
    [
        60, //FP_HEAL,//instant
        10, //FP_LEVITATION,//hold/duration
        50, //FP_SPEED,//duration
        20, //FP_PUSH,//hold/duration
        20, //FP_PULL,//hold/duration
        20, //FP_TELEPATHY,//instant
        30, //FP_GRIP,//hold/duration
        1,  //FP_LIGHTNING,//hold/duration
        50, //FP_RAGE,//duration
        25, //FP_PROTECT,//duration
        25, //FP_ABSORB,//duration
        33, //FP_TEAM_HEAL,//instant
        33, //FP_TEAM_FORCE,//instant
        20, //FP_DRAIN,//hold/duration
        20, //FP_SEE,//duration
        0,  //FP_SABER_OFFENSE,
        1,  //FP_SABER_DEFENSE,
        20, //FP_SABERTHROW,
    ],
    [
        50, //FP_HEAL,//instant //You get 5 points of health.. for 50 force points!
        10, //FP_LEVITATION,//hold/duration
        50, //FP_SPEED,//duration
        20, //FP_PUSH,//hold/duration
        20, //FP_PULL,//hold/duration
        20, //FP_TELEPATHY,//instant
        60, //FP_GRIP,//hold/duration
        1,  //FP_LIGHTNING,//hold/duration
        50, //FP_RAGE,//duration
        10, //FP_PROTECT,//duration
        10, //FP_ABSORB,//duration
        25, //FP_TEAM_HEAL,//instant
        25, //FP_TEAM_FORCE,//instant
        20, //FP_DRAIN,//hold/duration
        20, //FP_SEE,//duration
        0,  //FP_SABER_OFFENSE,
        0,  //FP_SABER_DEFENSE,
        20, //FP_SABERTHROW,
    ],
];
