#![allow(non_camel_case_types, non_snake_case)]

/// Raven `T_G_ICARUS_USE` — ICARUS `use` task-block arguments passed across
/// the ABI seam.
///
/// Type definition source: `oracle/codemp/game/g_public.h:866-870`
#[repr(C)]
pub struct T_G_ICARUS_USE {
    pub entID: i32,
    pub target: [i8; 2048],
}

const _: () = assert!(core::mem::size_of::<T_G_ICARUS_USE>() == 2052);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_USE, entID) == 0);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_USE, target) == 4);
