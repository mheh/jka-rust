#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ffi::c_int;

/// Raven `wpneighbor_t` (`wpneighbor_s`) — a waypoint neighbor link.
///
/// Type definition source: `oracle/oracle/codemp/game/q_shared.h:1001-1005`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct wpneighbor_t {
    pub num: c_int,
    pub forceJumpTo: c_int,
}

const _: () = {
    assert!(core::mem::size_of::<wpneighbor_t>() == 8);
};
