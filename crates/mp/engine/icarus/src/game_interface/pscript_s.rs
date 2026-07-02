#![allow(non_camel_case_types, non_snake_case)]

use std::os::raw::c_char;

/// Raven `pscript_s` — compiled Icarus script buffer handed to the game.
///
/// Raven: (none).
/// Type definition source: `oracle/oracle/codemp/icarus/GameInterface.h:4-8`
#[repr(C)]
pub struct pscript_t {
    pub buffer: *mut c_char,
    pub length: i64,
}

pub type pscript_s = pscript_t;

const _: () = assert!(core::mem::size_of::<pscript_t>() == 16);
const _: () = assert!(core::mem::offset_of!(pscript_t, buffer) == 0);
const _: () = assert!(core::mem::offset_of!(pscript_t, length) == 8);
