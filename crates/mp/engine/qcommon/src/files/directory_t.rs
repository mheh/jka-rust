#![allow(non_camel_case_types, non_snake_case)]

use std::os::raw::c_char;

// Raven `#define MAX_OSPATH PATH_MAX` (1024 here, matching this struct's field sizes).
// Source: oracle/oracle/codemp/game/q_shared.h:395
const MAX_OSPATH: usize = 1024;

/// Raven `directory_t` — a search path directory (base path + game subdirectory).
///
/// Raven: none.
/// Type definition source: `oracle/oracle/codemp/qcommon/files.h:58-61`
#[repr(C)]
pub struct directory_t {
	pub path: [c_char; MAX_OSPATH],    // c:\jk2
	pub gamedir: [c_char; MAX_OSPATH], // base
}

const _: () = assert!(core::mem::size_of::<directory_t>() == 2048);
const _: () = assert!(core::mem::offset_of!(directory_t, path) == 0);
const _: () = assert!(core::mem::offset_of!(directory_t, gamedir) == 1024);
