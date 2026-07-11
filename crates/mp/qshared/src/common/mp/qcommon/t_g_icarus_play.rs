#![allow(non_camel_case_types, non_snake_case)]

/// Raven `T_G_ICARUS_PLAY` — ICARUS `play` task-block arguments passed across
/// the ABI seam.
///
/// Type definition source: `oracle/codemp/game/g_public.h:884-890`
#[repr(C)]
pub struct T_G_ICARUS_PLAY {
    pub taskID: i32,
    pub entID: i32,
    pub r#type: [i8; 2048],
    pub name: [i8; 2048],
}

const _: () = assert!(core::mem::size_of::<T_G_ICARUS_PLAY>() == 4104);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_PLAY, taskID) == 0);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_PLAY, entID) == 4);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_PLAY, r#type) == 8);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_PLAY, name) == 2056);
