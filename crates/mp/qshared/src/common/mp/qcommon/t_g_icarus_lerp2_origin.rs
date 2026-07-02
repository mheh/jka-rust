#![allow(non_camel_case_types, non_snake_case)]

use crate::shared::vec3_t;

/// Raven `T_G_ICARUS_LERP2ORIGIN` — ICARUS `lerp2origin` task data passed
/// across the game ABI seam.
///
/// Type definition source: `oracle/oracle/codemp/game/g_public.h:828-834`
#[repr(C)]
pub struct T_G_ICARUS_LERP2ORIGIN {
	pub taskID: i32,
	pub entID: i32,
	pub origin: vec3_t,
	pub duration: f32,
}

const _: () = assert!(core::mem::size_of::<T_G_ICARUS_LERP2ORIGIN>() == 24);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_LERP2ORIGIN, taskID) == 0);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_LERP2ORIGIN, entID) == 4);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_LERP2ORIGIN, origin) == 8);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_LERP2ORIGIN, duration) == 20);
