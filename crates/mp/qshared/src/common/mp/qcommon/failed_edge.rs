//! MP `failedEdge_t` copied from Raven `codemp/game/g_public.h`.
//!
//! Source: `oracle/oracle/codemp/game/g_public.h:51-58`

#![allow(non_camel_case_types)]

use core::ffi::c_int;

/// This structure is shared by gameside and in-engine NPC nav routines.
///
/// Type definition source: `oracle/oracle/codemp/game/g_public.h:51-58`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct failedEdge_t {
    pub startID: c_int,
    pub endID: c_int,
    pub checkTime: c_int,
    pub entID: c_int,
}

const _: () = assert!(core::mem::size_of::<failedEdge_t>() == 16);
const _: () = assert!(core::mem::offset_of!(failedEdge_t, startID) == 0);
const _: () = assert!(core::mem::offset_of!(failedEdge_t, endID) == 4);
const _: () = assert!(core::mem::offset_of!(failedEdge_t, checkTime) == 8);
const _: () = assert!(core::mem::offset_of!(failedEdge_t, entID) == 12);
