//! MP `bg_public.h` saber attack quadrant definitions.
//!
//! Type definition source: `oracle/codemp/game/bg_public.h:1484-1494`

#![allow(non_camel_case_types)]

use std::os::raw::c_int;

/// Raven `saberQuadrant_t` — saber attack quadrant.
///
/// Raven: Enumeration defining the eight directional quadrants around a target
/// used for saber attack animations and blocking positions.
/// Type definition source: `oracle/codemp/game/bg_public.h:1484-1494`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum saberQuadrant_t {
    Q_BR = 0,
    Q_R = 1,
    Q_TR = 2,
    Q_T = 3,
    Q_TL = 4,
    Q_L = 5,
    Q_BL = 6,
    Q_B = 7,
    Q_NUM_QUADS = 8,
}

// Standalone const c_int equivalents for C interop (match arms, comparisons).
// Source: `oracle/codemp/game/bg_public.h:1484-1494`

/// Saber quadrant: bottom-right. Source: `oracle/codemp/game/bg_public.h:1485`
pub const Q_BR: c_int = 0;

/// Saber quadrant: right. Source: `oracle/codemp/game/bg_public.h:1486`
pub const Q_R: c_int = 1;

/// Saber quadrant: top-right. Source: `oracle/codemp/game/bg_public.h:1487`
pub const Q_TR: c_int = 2;

/// Saber quadrant: top. Source: `oracle/codemp/game/bg_public.h:1488`
pub const Q_T: c_int = 3;

/// Saber quadrant: top-left. Source: `oracle/codemp/game/bg_public.h:1489`
pub const Q_TL: c_int = 4;

/// Saber quadrant: left. Source: `oracle/codemp/game/bg_public.h:1490`
pub const Q_L: c_int = 5;

/// Saber quadrant: bottom-left. Source: `oracle/codemp/game/bg_public.h:1491`
pub const Q_BL: c_int = 6;

/// Saber quadrant: bottom. Source: `oracle/codemp/game/bg_public.h:1492`
pub const Q_B: c_int = 7;

/// Number of saber quadrants. Source: `oracle/codemp/game/bg_public.h:1493`
pub const Q_NUM_QUADS: c_int = 8;
