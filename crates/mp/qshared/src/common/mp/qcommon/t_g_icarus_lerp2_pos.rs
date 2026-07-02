#![allow(non_camel_case_types, non_snake_case)]

use crate::shared::{qboolean, vec3_t};

/// Raven `T_G_ICARUS_LERP2POS` — ICARUS `lerp2pos` task data passed across
/// the game ABI seam.
///
/// Type definition source: `oracle/oracle/codemp/game/g_public.h:818-826`
#[repr(C)]
pub struct T_G_ICARUS_LERP2POS {
	pub taskID: i32,
	pub entID: i32,
	pub origin: vec3_t,
	pub angles: vec3_t,
	pub duration: f32,
	/// special case
	pub nullAngles: qboolean,
}

const _: () = assert!(core::mem::size_of::<T_G_ICARUS_LERP2POS>() == 40);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_LERP2POS, taskID) == 0);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_LERP2POS, entID) == 4);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_LERP2POS, origin) == 8);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_LERP2POS, angles) == 20);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_LERP2POS, duration) == 32);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_LERP2POS, nullAngles) == 36);
