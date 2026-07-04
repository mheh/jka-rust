//! Raven `NPC_CheckVisibility` flag bits (`b_local.h`).
//!
//! Type definition source: `oracle/oracle/codemp/game/b_local.h:165-169`

#![allow(non_camel_case_types)]

use core::ffi::c_int;

/// Raven `CHECK_PVS` — visibility-check flag bit.
///
/// Source: `oracle/oracle/codemp/game/b_local.h:165`
pub const CHECK_PVS: c_int = 1;
/// Raven `CHECK_360` — visibility-check flag bit.
///
/// Source: `oracle/oracle/codemp/game/b_local.h:166`
pub const CHECK_360: c_int = 2;
/// Raven `CHECK_FOV` — visibility-check flag bit.
///
/// Source: `oracle/oracle/codemp/game/b_local.h:167`
pub const CHECK_FOV: c_int = 4;
/// Raven `CHECK_SHOOT` — visibility-check flag bit.
///
/// Source: `oracle/oracle/codemp/game/b_local.h:168`
pub const CHECK_SHOOT: c_int = 8;
/// Raven `CHECK_VISRANGE` — visibility-check flag bit.
///
/// Source: `oracle/oracle/codemp/game/b_local.h:169`
pub const CHECK_VISRANGE: c_int = 16;
