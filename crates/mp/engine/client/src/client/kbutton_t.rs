#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::qboolean;

/// Raven `kbutton_t` — button/key hold-time tracking state.
///
/// Raven: (field comments) `down` = key nums holding it down; `downtime` = msec
/// timestamp; `msec` = msec down this frame if both a down and up happened;
/// `active` = current state; `wasPressed` = set when down, not cleared when up.
/// Type definition source: `oracle/oracle/codemp/client/client.h:479-485`
#[repr(C)]
pub struct kbutton_t {
    pub down: [i32; 2],
    pub downtime: u32,
    pub msec: u32,
    pub active: qboolean,
    pub wasPressed: qboolean,
}

const _: () = assert!(core::mem::size_of::<kbutton_t>() == 24);
const _: () = assert!(core::mem::offset_of!(kbutton_t, down) == 0);
const _: () = assert!(core::mem::offset_of!(kbutton_t, downtime) == 8);
const _: () = assert!(core::mem::offset_of!(kbutton_t, msec) == 12);
const _: () = assert!(core::mem::offset_of!(kbutton_t, active) == 16);
const _: () = assert!(core::mem::offset_of!(kbutton_t, wasPressed) == 20);
