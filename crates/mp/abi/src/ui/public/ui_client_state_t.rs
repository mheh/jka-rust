#![allow(non_camel_case_types, non_snake_case)]

use std::os::raw::c_char;

use mp_qshared::shared::{connstate_t, MAX_STRING_CHARS};

/// Raven `uiClientState_t` — client connection state as seen by the UI module.
///
/// Type definition source: `oracle/oracle/codemp/ui/ui_public.h:8-15`
#[repr(C)]
pub struct uiClientState_t {
    pub connState: connstate_t,
    pub connectPacketCount: i32,
    pub clientNum: i32,
    pub servername: [c_char; MAX_STRING_CHARS],
    pub updateInfoString: [c_char; MAX_STRING_CHARS],
    pub messageString: [c_char; MAX_STRING_CHARS],
}

const _: () = assert!(core::mem::size_of::<uiClientState_t>() == 3084);
const _: () = assert!(core::mem::offset_of!(uiClientState_t, connState) == 0);
const _: () = assert!(core::mem::offset_of!(uiClientState_t, connectPacketCount) == 4);
const _: () = assert!(core::mem::offset_of!(uiClientState_t, clientNum) == 8);
const _: () = assert!(core::mem::offset_of!(uiClientState_t, servername) == 12);
const _: () = assert!(core::mem::offset_of!(uiClientState_t, updateInfoString) == 1036);
const _: () = assert!(core::mem::offset_of!(uiClientState_t, messageString) == 2060);
