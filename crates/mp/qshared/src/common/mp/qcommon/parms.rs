//! MP `parms_t` copied from Raven `codemp/game/g_public.h`.
//!
//! Source: `oracle/oracle/codemp/game/g_public.h:666-671`

#![allow(non_camel_case_types)]

use core::ffi::c_char;

pub const MAX_PARMS: usize = 16;
pub const MAX_PARM_STRING_LENGTH: usize = crate::shared::MAX_QPATH;

/// Raven `parms_t`.
///
/// Type definition source: `oracle/oracle/codemp/game/g_public.h:666-671`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct parms_t {
    pub parm: [[c_char; MAX_PARM_STRING_LENGTH]; MAX_PARMS],
}

const _: () = assert!(core::mem::size_of::<parms_t>() == 1024);
const _: () = assert!(core::mem::offset_of!(parms_t, parm) == 0);
