#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::shared::vec3_t;

/// Raven `move_rotate_t` — per-frame ROFF delta origin/rotation entry.
///
/// Type definition source: `oracle/code/game/g_roff.h:31-36`
#[repr(C)]
pub struct move_rotate_t {
	pub origin_delta: vec3_t,
	pub rotate_delta: vec3_t,
}

const _: () = assert!(core::mem::size_of::<move_rotate_t>() == 24);
const _: () = assert!(core::mem::offset_of!(move_rotate_t, origin_delta) == 0);
const _: () = assert!(core::mem::offset_of!(move_rotate_t, rotate_delta) == 12);
