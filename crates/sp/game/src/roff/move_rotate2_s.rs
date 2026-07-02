#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::shared::vec3_t;

/// Raven `move_rotate2_t` — per-frame ROFF delta origin/rotation entry with
/// per-vertex note-track range.
///
/// Type definition source: `oracle/oracle/code/game/g_roff.h:50-57`
#[repr(C)]
pub struct move_rotate2_t {
	pub origin_delta: vec3_t,
	pub rotate_delta: vec3_t,
	// note track info
	pub mStartNote: i32,
	pub mNumNotes: i32,
}

const _: () = assert!(core::mem::size_of::<move_rotate2_t>() == 32);
const _: () = assert!(core::mem::offset_of!(move_rotate2_t, origin_delta) == 0);
const _: () = assert!(core::mem::offset_of!(move_rotate2_t, rotate_delta) == 12);
const _: () = assert!(core::mem::offset_of!(move_rotate2_t, mStartNote) == 24);
const _: () = assert!(core::mem::offset_of!(move_rotate2_t, mNumNotes) == 28);
