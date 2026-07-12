#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

/// Raven `T_G_ICARUS_REMOVE` — ICARUS entity-removal command payload.
///
/// Type definition source: `oracle/codemp/game/g_public.h:878-882`
#[repr(C)]
pub struct T_G_ICARUS_REMOVE {
    pub entID: i32,
    pub name: [c_char; 2048],
}

const _: () = assert!(core::mem::size_of::<T_G_ICARUS_REMOVE>() == 2052);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_REMOVE, entID) == 0);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_REMOVE, name) == 4);
