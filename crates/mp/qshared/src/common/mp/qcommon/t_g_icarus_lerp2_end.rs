#![allow(non_camel_case_types, non_snake_case)]

/// Raven `T_G_ICARUS_LERP2END` — ICARUS `lerp2end` task data passed across
/// the game ABI seam.
///
/// Type definition source: `oracle/oracle/codemp/game/g_public.h:859-864`
#[repr(C)]
pub struct T_G_ICARUS_LERP2END {
	pub entID: i32,
	pub taskID: i32,
	pub duration: f32,
}

const _: () = assert!(core::mem::size_of::<T_G_ICARUS_LERP2END>() == 12);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_LERP2END, entID) == 0);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_LERP2END, taskID) == 4);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_LERP2END, duration) == 8);
