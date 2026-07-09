#![allow(non_camel_case_types, non_snake_case)]

/// Raven `T_G_ICARUS_PLAYSOUND` — ICARUS task args for `G_ICARUS_PLAYSOUND`.
///
/// Type definition source: `oracle/codemp/game/g_public.h:801-807`
#[repr(C)]
pub struct T_G_ICARUS_PLAYSOUND {
	pub taskID: i32,
	pub entID: i32,
	pub name: [u8; 2048],
	pub channel: [u8; 2048],
}

const _: () = assert!(core::mem::size_of::<T_G_ICARUS_PLAYSOUND>() == 4104);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_PLAYSOUND, taskID) == 0);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_PLAYSOUND, entID) == 4);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_PLAYSOUND, name) == 8);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_PLAYSOUND, channel) == 2056);
