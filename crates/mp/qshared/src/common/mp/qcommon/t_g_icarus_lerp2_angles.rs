#![allow(non_camel_case_types, non_snake_case)]

use crate::shared::vec3_t;

/// Raven `T_G_ICARUS_LERP2ANGLES` — ICARUS `lerp2angles` task data passed
/// across the game ABI seam.
///
/// Type definition source: `oracle/oracle/codemp/game/g_public.h:836-842`
#[repr(C)]
pub struct T_G_ICARUS_LERP2ANGLES {
	pub taskID: i32,
	pub entID: i32,
	pub angles: vec3_t,
	pub duration: f32,
}

const _: () = assert!(core::mem::size_of::<T_G_ICARUS_LERP2ANGLES>() == 24);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_LERP2ANGLES, taskID) == 0);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_LERP2ANGLES, entID) == 4);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_LERP2ANGLES, angles) == 8);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_LERP2ANGLES, duration) == 20);
