//! MP `bg_public.h` saber attack quadrant definitions.
//!
//! Type definition source: `oracle/oracle/codemp/game/bg_public.h:1484-1494`

#![allow(non_camel_case_types)]

/// Raven `saberQuadrant_t` — saber attack quadrant.
///
/// Raven: Enumeration defining the eight directional quadrants around a target
/// used for saber attack animations and blocking positions.
/// Type definition source: `oracle/oracle/codemp/game/bg_public.h:1484-1494`
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
