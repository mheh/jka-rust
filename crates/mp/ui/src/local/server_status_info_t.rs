#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;
use mp_qshared::shared::MAX_CLIENTS;

/// `MAX_ADDRESSLENGTH`.
///
/// Source: `oracle/oracle/codemp/ui/ui_local.h:571`
const MAX_ADDRESSLENGTH: usize = 64;

/// `MAX_SERVERSTATUS_LINES`.
///
/// Source: `oracle/oracle/codemp/ui/ui_local.h:578`
const MAX_SERVERSTATUS_LINES: usize = 128;

/// `MAX_SERVERSTATUS_TEXT`.
///
/// Source: `oracle/oracle/codemp/ui/ui_local.h:579`
const MAX_SERVERSTATUS_TEXT: usize = 1024;

/// Raven `serverStatusInfo_t`.
///
/// Type definition source: `oracle/oracle/codemp/ui/ui_local.h:703-709`
#[repr(C)]
pub struct serverStatusInfo_t {
	pub address: [c_char; MAX_ADDRESSLENGTH],
	pub lines: [[*mut c_char; 4]; MAX_SERVERSTATUS_LINES],
	pub text: [c_char; MAX_SERVERSTATUS_TEXT],
	pub pings: [c_char; MAX_CLIENTS * 3],
	pub numLines: i32,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<serverStatusInfo_t>() == 5288);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(serverStatusInfo_t, address) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(serverStatusInfo_t, lines) == 64);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(serverStatusInfo_t, text) == 4160);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(serverStatusInfo_t, pings) == 5184);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(serverStatusInfo_t, numLines) == 5280);
