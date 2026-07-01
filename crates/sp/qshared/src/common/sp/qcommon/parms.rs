//! SP `parms_t` copied from Raven `code/game/g_shared.h`.
//!
//! Source: `oracle/oracle/code/game/g_shared.h:490-495`

#![allow(non_camel_case_types)]

use core::ffi::c_char;

pub const MAX_PARMS: usize = 16;
pub const MAX_PARM_STRING_LENGTH: usize = crate::shared::MAX_QPATH;

/// Raven `parms_t`.
///
/// Type definition source: `oracle/oracle/code/game/g_shared.h:490-495`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct parms_t {
    pub parm: [[c_char; MAX_PARM_STRING_LENGTH]; MAX_PARMS],
}
