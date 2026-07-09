#![allow(non_camel_case_types, non_snake_case)]

/// Raven `powerupInfo_t` — cgame-side powerup registration entry.
///
/// Type definition source: `oracle/codemp/cgame/cg_local.h:723-725`
#[repr(C)]
pub struct powerupInfo_t {
	pub itemNum: i32,
}

const _: () = assert!(core::mem::size_of::<powerupInfo_t>() == 4);
const _: () = assert!(core::mem::offset_of!(powerupInfo_t, itemNum) == 0);
