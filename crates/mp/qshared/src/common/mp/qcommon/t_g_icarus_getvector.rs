#![allow(non_camel_case_types, non_snake_case)]

use crate::shared::vec3_t;

/// Raven `T_G_ICARUS_GETVECTOR` — ICARUS `getvector` task data passed across
/// the game ABI seam.
///
/// Type definition source: `oracle/codemp/game/g_public.h:900-906`
#[repr(C)]
pub struct T_G_ICARUS_GETVECTOR {
    pub entID: i32,
    pub r#type: i32,
    pub name: [u8; 2048],
    pub value: vec3_t,
}

const _: () = assert!(core::mem::size_of::<T_G_ICARUS_GETVECTOR>() == 2068);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_GETVECTOR, entID) == 0);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_GETVECTOR, r#type) == 4);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_GETVECTOR, name) == 8);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_GETVECTOR, value) == 2056);
