#![allow(non_camel_case_types, non_snake_case)]

/// Raven `T_G_ICARUS_KILL` — ICARUS `KILL` command payload (entity to kill, by
/// id or by name).
///
/// Type definition source: `oracle/codemp/game/g_public.h:872-876`
#[repr(C)]
pub struct T_G_ICARUS_KILL {
	pub entID: i32,
	pub name: [u8; 2048],
}

const _: () = assert!(core::mem::size_of::<T_G_ICARUS_KILL>() == 2052);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_KILL, entID) == 0);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_KILL, name) == 4);
