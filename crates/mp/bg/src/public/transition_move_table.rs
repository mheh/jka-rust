//! MP `bg_saber.c` quadrant-to-quadrant transition-move lookup.
//!
//! Source: `oracle/codemp/game/bg_saber.c:324-390`

#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use core::ffi::c_int;

use super::saber_move_name::*;
use super::saber_quadrant::saberQuadrant_t::Q_NUM_QUADS;

/// Raven `transitionMove[Q_NUM_QUADS][Q_NUM_QUADS]` — for a `[from-quad][to-quad]`
/// pair, the `LS_T1_*` transition move that bridges them (`LS_NONE` where no
/// transition exists). Row-major, matching the C flat initializer.
///
/// Source: `oracle/codemp/game/bg_saber.c:324-390`
#[rustfmt::skip]
pub static transitionMove: [[c_int; Q_NUM_QUADS as usize]; Q_NUM_QUADS as usize] = [
    [
        LS_NONE, //Can't transition to same pos!
        LS_T1_BR__R, //40
        LS_T1_BR_TR,
        LS_T1_BR_T_,
        LS_T1_BR_TL,
        LS_T1_BR__L,
        LS_T1_BR_BL,
        LS_NONE, //No transitions to bottom, and no anims start there, so shouldn't need any
    ],
    [
        LS_T1__R_BR, //46
        LS_NONE, //Can't transition to same pos!
        LS_T1__R_TR,
        LS_T1__R_T_,
        LS_T1__R_TL,
        LS_T1__R__L,
        LS_T1__R_BL,
        LS_NONE, //No transitions to bottom, and no anims start there, so shouldn't need any
    ],
    [
        LS_T1_TR_BR, //52
        LS_T1_TR__R,
        LS_NONE, //Can't transition to same pos!
        LS_T1_TR_T_,
        LS_T1_TR_TL,
        LS_T1_TR__L,
        LS_T1_TR_BL,
        LS_NONE, //No transitions to bottom, and no anims start there, so shouldn't need any
    ],
    [
        LS_T1_T__BR, //58
        LS_T1_T___R,
        LS_T1_T__TR,
        LS_NONE, //Can't transition to same pos!
        LS_T1_T__TL,
        LS_T1_T___L,
        LS_T1_T__BL,
        LS_NONE, //No transitions to bottom, and no anims start there, so shouldn't need any
    ],
    [
        LS_T1_TL_BR, //64
        LS_T1_TL__R,
        LS_T1_TL_TR,
        LS_T1_TL_T_,
        LS_NONE, //Can't transition to same pos!
        LS_T1_TL__L,
        LS_T1_TL_BL,
        LS_NONE, //No transitions to bottom, and no anims start there, so shouldn't need any
    ],
    [
        LS_T1__L_BR, //70
        LS_T1__L__R,
        LS_T1__L_TR,
        LS_T1__L_T_,
        LS_T1__L_TL,
        LS_NONE, //Can't transition to same pos!
        LS_T1__L_BL,
        LS_NONE, //No transitions to bottom, and no anims start there, so shouldn't need any
    ],
    [
        LS_T1_BL_BR, //76
        LS_T1_BL__R,
        LS_T1_BL_TR,
        LS_T1_BL_T_,
        LS_T1_BL_TL,
        LS_T1_BL__L,
        LS_NONE, //Can't transition to same pos!
        LS_NONE, //No transitions to bottom, and no anims start there, so shouldn't need any
    ],
    [
        LS_T1_BL_BR, //NOTE: there are no transitions from bottom, so re-use the bottom right transitions
        LS_T1_BR__R,
        LS_T1_BR_TR,
        LS_T1_BR_T_,
        LS_T1_BR_TL,
        LS_T1_BR__L,
        LS_T1_BR_BL,
        LS_NONE, //No transitions to bottom, and no anims start there, so shouldn't need any
    ],
];
