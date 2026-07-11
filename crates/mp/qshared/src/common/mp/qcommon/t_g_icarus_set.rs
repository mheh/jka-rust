#![allow(non_camel_case_types, non_snake_case)]

/// Raven `T_G_ICARUS_SET` — ICARUS `set` command payload passed across the
/// game/engine boundary.
///
/// Type definition source: `oracle/codemp/game/g_public.h:810-816`
#[repr(C)]
pub struct T_G_ICARUS_SET {
    pub taskID: i32,
    pub entID: i32,
    pub type_name: [i8; 2048],
    pub data: [i8; 2048],
}

const _: () = assert!(core::mem::size_of::<T_G_ICARUS_SET>() == 4104);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_SET, taskID) == 0);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_SET, entID) == 4);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_SET, type_name) == 8);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_SET, data) == 2056);
