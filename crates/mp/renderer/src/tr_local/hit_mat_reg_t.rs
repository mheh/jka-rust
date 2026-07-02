#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use mp_qshared::shared::MAX_QPATH;

/// Raven `hitMatReg_t` — hit-material registration entry (raw data blob,
/// dimensions, source name).
///
/// Type definition source: `oracle/oracle/codemp/renderer/tr_local.h:544-550`
#[repr(C)]
pub struct hitMatReg_t {
    pub loc: *mut u8,
    pub width: i32,
    pub height: i32,
    pub name: [c_char; MAX_QPATH as usize],
}

const _: () = assert!(core::mem::size_of::<hitMatReg_t>() == 80);
const _: () = assert!(core::mem::offset_of!(hitMatReg_t, loc) == 0);
const _: () = assert!(core::mem::offset_of!(hitMatReg_t, width) == 8);
const _: () = assert!(core::mem::offset_of!(hitMatReg_t, height) == 12);
const _: () = assert!(core::mem::offset_of!(hitMatReg_t, name) == 16);
