#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use mp_qshared::shared::qboolean;

/// Raven `pendingServer_t`.
///
/// Type definition source: `oracle/oracle/codemp/ui/ui_local.h:690-696`
#[repr(C)]
pub struct pendingServer_t {
	pub adrstr: [c_char; 64],
	pub name: [c_char; 64],
	pub startTime: i32,
	pub serverNum: i32,
	pub valid: qboolean,
}

const _: () = assert!(core::mem::size_of::<pendingServer_t>() == 140);
const _: () = assert!(core::mem::offset_of!(pendingServer_t, adrstr) == 0);
const _: () = assert!(core::mem::offset_of!(pendingServer_t, name) == 64);
const _: () = assert!(core::mem::offset_of!(pendingServer_t, startTime) == 128);
const _: () = assert!(core::mem::offset_of!(pendingServer_t, serverNum) == 132);
const _: () = assert!(core::mem::offset_of!(pendingServer_t, valid) == 136);
