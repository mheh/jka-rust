#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

/// Raven `T_G_ICARUS_GETSTRING` — ICARUS `G_ICARUS_GetString` in/out param block.
///
/// Type definition source: `oracle/oracle/codemp/game/g_public.h:908-914`
#[repr(C)]
pub struct T_G_ICARUS_GETSTRING {
    pub entID: c_int,
    pub r#type: c_int,
    pub name: [u8; 2048],
    pub value: [u8; 2048],
}

const _: () = assert!(core::mem::size_of::<T_G_ICARUS_GETSTRING>() == 4104);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_GETSTRING, entID) == 0);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_GETSTRING, r#type) == 4);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_GETSTRING, name) == 8);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_GETSTRING, value) == 2056);
