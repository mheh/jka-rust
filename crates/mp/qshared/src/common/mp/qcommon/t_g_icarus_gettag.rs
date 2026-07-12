#![allow(non_camel_case_types, non_snake_case)]

use crate::shared::vec3_t;

/// Raven `T_G_ICARUS_GETTAG` — ICARUS `GETTAG` command payload.
///
/// Type definition source: `oracle/codemp/game/g_public.h:844-850`
#[repr(C)]
pub struct T_G_ICARUS_GETTAG {
    pub entID: i32,
    pub name: [u8; 2048],
    pub lookup: i32,
    pub info: vec3_t,
}

const _: () = assert!(core::mem::size_of::<T_G_ICARUS_GETTAG>() == 2068);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_GETTAG, entID) == 0);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_GETTAG, name) == 4);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_GETTAG, lookup) == 2052);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_GETTAG, info) == 2056);
