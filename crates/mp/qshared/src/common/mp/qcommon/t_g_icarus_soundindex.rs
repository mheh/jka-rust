#![allow(non_camel_case_types, non_snake_case)]

/// Raven `T_G_ICARUS_SOUNDINDEX` — ICARUS sound-index registration args.
///
/// Type definition source: `oracle/codemp/game/g_public.h:916-919`
#[repr(C)]
pub struct T_G_ICARUS_SOUNDINDEX {
    pub filename: [core::ffi::c_char; 2048],
}

const _: () = assert!(core::mem::size_of::<T_G_ICARUS_SOUNDINDEX>() == 2048);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_SOUNDINDEX, filename) == 0);
