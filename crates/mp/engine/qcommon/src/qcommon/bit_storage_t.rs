#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

/// Raven `bitStorage_t` (`struct bitStorage_s`) — one node of the singly-linked
/// list that saves off the default `bits` of each `entityStateFields`/
/// `playerStateFields` entry so `MSG_CheckNETFPSFOverrides` can restore them
/// before re-applying a mod's override file.
///
/// C-faithful shape (raw `next` pointer, Z_Malloc-owned) per the pre-safe-state
/// migration convention; only defined under `!_XBOX` (mods disabled on Xbox).
/// Type definition source: `oracle/codemp/qcommon/msg.cpp:1994-2000`
pub struct bitStorage_t {
    pub next: *mut bitStorage_t,
    pub bits: c_int,
}
