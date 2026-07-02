#![allow(non_camel_case_types, non_snake_case)]

/// Raven `T_G_ICARUS_LERP2START` — ICARUS lerp-to-start task args for an entity.
///
/// Type definition source: `oracle/oracle/codemp/game/g_public.h:852-857`
#[repr(C)]
pub struct T_G_ICARUS_LERP2START {
    pub entID: i32,
    pub taskID: i32,
    pub duration: f32,
}

const _: () = assert!(core::mem::size_of::<T_G_ICARUS_LERP2START>() == 12);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_LERP2START, entID) == 0);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_LERP2START, taskID) == 4);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_LERP2START, duration) == 8);
