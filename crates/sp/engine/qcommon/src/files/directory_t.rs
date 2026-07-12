#![allow(non_camel_case_types, non_snake_case)]

use std::os::raw::c_char;

// Raven `#define MAX_OSPATH 260` (max length of a filesystem pathname).
// Source: oracle/code/game/q_shared.h:216
const MAX_OSPATH: usize = 260;

/// Raven `directory_t` — a search path directory (base path + game subdirectory).
///
/// Raven: none.
/// Type definition source: `oracle/code/qcommon/files.h:45-48`
#[repr(C)]
pub struct directory_t {
    pub path: [c_char; MAX_OSPATH],    // c:\stvoy
    pub gamedir: [c_char; MAX_OSPATH], // base
}

const _: () = assert!(core::mem::size_of::<directory_t>() == 520);
const _: () = assert!(core::mem::offset_of!(directory_t, path) == 0);
const _: () = assert!(core::mem::offset_of!(directory_t, gamedir) == 260);
