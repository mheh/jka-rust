#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use mp_qshared::shared::{qboolean, qhandle_t};

/// `MAX_GAMETYPES`.
///
/// Source: `oracle/oracle/codemp/ui/ui_local.h:566`
const MAX_GAMETYPES: usize = 16;

/// Raven `mapInfo`.
///
/// Type definition source: `oracle/oracle/codemp/ui/ui_local.h:629-640`
#[repr(C)]
pub struct mapInfo {
	pub mapName: *const c_char,
	pub mapLoadName: *const c_char,
	pub imageName: *const c_char,
	pub opponentName: *const c_char,
	pub teamMembers: i32,
	pub typeBits: i32,
	pub cinematic: i32,
	pub timeToBeat: [i32; MAX_GAMETYPES],
	pub levelShot: qhandle_t,
	pub active: qboolean,
}

const _: () = assert!(core::mem::size_of::<mapInfo>() == 120);
const _: () = assert!(core::mem::offset_of!(mapInfo, mapName) == 0);
const _: () = assert!(core::mem::offset_of!(mapInfo, mapLoadName) == 8);
const _: () = assert!(core::mem::offset_of!(mapInfo, imageName) == 16);
const _: () = assert!(core::mem::offset_of!(mapInfo, opponentName) == 24);
const _: () = assert!(core::mem::offset_of!(mapInfo, teamMembers) == 32);
const _: () = assert!(core::mem::offset_of!(mapInfo, typeBits) == 36);
const _: () = assert!(core::mem::offset_of!(mapInfo, cinematic) == 40);
const _: () = assert!(core::mem::offset_of!(mapInfo, timeToBeat) == 44);
const _: () = assert!(core::mem::offset_of!(mapInfo, levelShot) == 108);
const _: () = assert!(core::mem::offset_of!(mapInfo, active) == 112);
