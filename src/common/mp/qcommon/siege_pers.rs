//! MP `siegePers_t` copied from Raven `codemp/game/q_shared.h`.
//!
//! Source: `oracle/oracle/codemp/game/q_shared.h:2437-2442`

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use crate::shared::qboolean;

/// Raven `siegePers_t` persistent Siege state.
///
/// Type definition source: `oracle/oracle/codemp/game/q_shared.h:2437-2442`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct siegePers_t {
    pub beatingTime: qboolean,
    pub lastTeam: c_int,
    pub lastTime: c_int,
}
