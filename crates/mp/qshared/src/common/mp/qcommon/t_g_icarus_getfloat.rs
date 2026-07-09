#![allow(non_camel_case_types, non_snake_case)]

/// Raven `T_G_ICARUS_GETFLOAT` — ICARUS `getfloat` task data passed across
/// the game ABI seam.
///
/// Type definition source: `oracle/codemp/game/g_public.h:892-898`
#[repr(C)]
pub struct T_G_ICARUS_GETFLOAT {
	pub entID: i32,
	pub r#type: i32,
	pub name: [u8; 2048],
	pub value: f32,
}

const _: () = assert!(core::mem::size_of::<T_G_ICARUS_GETFLOAT>() == 2060);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_GETFLOAT, entID) == 0);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_GETFLOAT, r#type) == 4);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_GETFLOAT, name) == 8);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_GETFLOAT, value) == 2056);
