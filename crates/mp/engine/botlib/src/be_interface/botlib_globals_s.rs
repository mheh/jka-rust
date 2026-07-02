#![allow(non_camel_case_types, non_snake_case)]

/// Raven `botlib_globals_t` — bot library global state.
///
/// Raven: `botlibsetup` true when the bot library has been setup;
/// `maxentities` maximum number of entities; `maxclients` maximum number of
/// clients; `time` the global time. The `#ifdef DEBUG` fields (`debug`,
/// `goalareanum`, `goalorigin`, `runai`) are compiled out of release builds.
/// Type definition source: `oracle/oracle/codemp/botlib/be_interface.h:19-31`
#[repr(C)]
pub struct botlib_globals_t {
    pub botlibsetup: i32,
    pub maxentities: i32,
    pub maxclients: i32,
    pub time: f32,
}

pub type botlib_globals_s = botlib_globals_t;

const _: () = assert!(core::mem::size_of::<botlib_globals_t>() == 16);
const _: () = assert!(core::mem::offset_of!(botlib_globals_t, botlibsetup) == 0);
const _: () = assert!(core::mem::offset_of!(botlib_globals_t, maxentities) == 4);
const _: () = assert!(core::mem::offset_of!(botlib_globals_t, maxclients) == 8);
const _: () = assert!(core::mem::offset_of!(botlib_globals_t, time) == 12);
