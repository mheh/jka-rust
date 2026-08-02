#![allow(non_camel_case_types, non_snake_case)]

//! `cm_terrainmap.h` constants: the automap image size and the team side codes.
//!
//! Source: `oracle/codemp/qcommon/cm_terrainmap.h:6-15,62-67`

use core::ffi::c_int;

// The `_XBOX` arm of the header selects 64/64/4. This port keeps the PC arm.
// Source: `oracle/codemp/qcommon/cm_terrainmap.h:5-13`

/// Raven `TM_WIDTH` — the automap image width in pixels.
/// Source: `oracle/codemp/qcommon/cm_terrainmap.h:10`
pub const TM_WIDTH: c_int = 512;

/// Raven `TM_HEIGHT` — the automap image height in pixels.
/// Source: `oracle/codemp/qcommon/cm_terrainmap.h:11`
pub const TM_HEIGHT: c_int = 512;

/// Raven `TM_BORDER` — the blank margin around the drawn area.
/// Source: `oracle/codemp/qcommon/cm_terrainmap.h:12`
pub const TM_BORDER: c_int = 16;

/// Raven `TM_REAL_WIDTH` — the drawn width inside the border.
/// Source: `oracle/codemp/qcommon/cm_terrainmap.h:14`
pub const TM_REAL_WIDTH: c_int = TM_WIDTH - TM_BORDER - TM_BORDER;

/// Raven `TM_REAL_HEIGHT` — the drawn height inside the border.
/// Source: `oracle/codemp/qcommon/cm_terrainmap.h:15`
pub const TM_REAL_HEIGHT: c_int = TM_HEIGHT - TM_BORDER - TM_BORDER;

// Raven declares the side codes as an anonymous enum, so they stay `c_int`
// constants and not a named Rust enum.
// Source: `oracle/codemp/qcommon/cm_terrainmap.h:62-67`

/// Raven `SIDE_NONE` — the marker belongs to no team.
/// Source: `oracle/codemp/qcommon/cm_terrainmap.h:64`
pub const SIDE_NONE: c_int = 0;

/// Raven `SIDE_BLUE` — the marker belongs to the blue team.
/// Source: `oracle/codemp/qcommon/cm_terrainmap.h:65`
pub const SIDE_BLUE: c_int = 1;

/// Raven `SIDE_RED` — the marker belongs to the red team.
/// Source: `oracle/codemp/qcommon/cm_terrainmap.h:66`
pub const SIDE_RED: c_int = 2;
