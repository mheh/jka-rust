//! MP `siegePers_t` copied from Raven `codemp/game/q_shared.h`.
//!
//! Source: `oracle/codemp/game/q_shared.h:2437-2442`

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use crate::shared::qboolean;

/// Raven `siegePers_t` persistent Siege state.
///
/// Type definition source: `oracle/codemp/game/q_shared.h:2437-2442`
#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct siegePers_t {
    pub beatingTime: qboolean,
    pub lastTeam: c_int,
    pub lastTime: c_int,
}

const _: () = assert!(core::mem::size_of::<siegePers_t>() == 12);
const _: () = assert!(core::mem::offset_of!(siegePers_t, beatingTime) == 0);
const _: () = assert!(core::mem::offset_of!(siegePers_t, lastTeam) == 4);
const _: () = assert!(core::mem::offset_of!(siegePers_t, lastTime) == 8);
