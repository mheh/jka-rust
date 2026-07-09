#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::shared::qboolean;

/// Raven `kbutton_t` — tracks a button-bound key's press state across a frame.
///
/// Raven: (unnamed).
/// Type definition source: `oracle/code/client/client.h:332-338`
#[repr(C)]
pub struct kbutton_t {
    /// key nums holding it down
    pub down: [i32; 2],
    /// msec timestamp
    pub downtime: u32,
    /// msec down this frame if both a down and up happened
    pub msec: u32,
    /// current state
    pub active: qboolean,
    /// set when down, not cleared when up
    pub wasPressed: qboolean,
}

const _: () = assert!(core::mem::size_of::<kbutton_t>() == 24);
const _: () = assert!(core::mem::offset_of!(kbutton_t, down) == 0);
const _: () = assert!(core::mem::offset_of!(kbutton_t, downtime) == 8);
const _: () = assert!(core::mem::offset_of!(kbutton_t, msec) == 12);
const _: () = assert!(core::mem::offset_of!(kbutton_t, active) == 16);
const _: () = assert!(core::mem::offset_of!(kbutton_t, wasPressed) == 20);
