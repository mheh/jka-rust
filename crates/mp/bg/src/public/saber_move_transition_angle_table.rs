//! MP `bg_saber.c` quadrant-to-quadrant transition-angle lookup.
//!
//! Source: `oracle/codemp/game/bg_saber.c:715-781`

#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use core::ffi::c_int;

use super::saber_quadrant::saberQuadrant_t::Q_NUM_QUADS;

/// Raven `saberMoveTransitionAngle[Q_NUM_QUADS][Q_NUM_QUADS]` — for a
/// `[from-quad][to-quad]` pair, the angle (degrees) swept by the transition.
/// Row-major, matching the C flat initializer.
///
/// Source: `oracle/codemp/game/bg_saber.c:715-781`
#[rustfmt::skip]
pub static saberMoveTransitionAngle: [[c_int; Q_NUM_QUADS as usize]; Q_NUM_QUADS as usize] = [
    [
        0, //Q_BR,Q_BR,
        45, //Q_BR,Q_R,
        90, //Q_BR,Q_TR,
        135, //Q_BR,Q_T,
        180, //Q_BR,Q_TL,
        215, //Q_BR,Q_L,
        270, //Q_BR,Q_BL,
        45, //Q_BR,Q_B,
    ],
    [
        45, //Q_R,Q_BR,
        0, //Q_R,Q_R,
        45, //Q_R,Q_TR,
        90, //Q_R,Q_T,
        135, //Q_R,Q_TL,
        180, //Q_R,Q_L,
        215, //Q_R,Q_BL,
        90, //Q_R,Q_B,
    ],
    [
        90, //Q_TR,Q_BR,
        45, //Q_TR,Q_R,
        0, //Q_TR,Q_TR,
        45, //Q_TR,Q_T,
        90, //Q_TR,Q_TL,
        135, //Q_TR,Q_L,
        180, //Q_TR,Q_BL,
        135, //Q_TR,Q_B,
    ],
    [
        135, //Q_T,Q_BR,
        90, //Q_T,Q_R,
        45, //Q_T,Q_TR,
        0, //Q_T,Q_T,
        45, //Q_T,Q_TL,
        90, //Q_T,Q_L,
        135, //Q_T,Q_BL,
        180, //Q_T,Q_B,
    ],
    [
        180, //Q_TL,Q_BR,
        135, //Q_TL,Q_R,
        90, //Q_TL,Q_TR,
        45, //Q_TL,Q_T,
        0, //Q_TL,Q_TL,
        45, //Q_TL,Q_L,
        90, //Q_TL,Q_BL,
        135, //Q_TL,Q_B,
    ],
    [
        215, //Q_L,Q_BR,
        180, //Q_L,Q_R,
        135, //Q_L,Q_TR,
        90, //Q_L,Q_T,
        45, //Q_L,Q_TL,
        0, //Q_L,Q_L,
        45, //Q_L,Q_BL,
        90, //Q_L,Q_B,
    ],
    [
        270, //Q_BL,Q_BR,
        215, //Q_BL,Q_R,
        180, //Q_BL,Q_TR,
        135, //Q_BL,Q_T,
        90, //Q_BL,Q_TL,
        45, //Q_BL,Q_L,
        0, //Q_BL,Q_BL,
        45, //Q_BL,Q_B,
    ],
    [
        45, //Q_B,Q_BR,
        90, //Q_B,Q_R,
        135, //Q_B,Q_TR,
        180, //Q_B,Q_T,
        135, //Q_B,Q_TL,
        90, //Q_B,Q_L,
        45, //Q_B,Q_BL,
        0, //Q_B,Q_B,
    ],
];
