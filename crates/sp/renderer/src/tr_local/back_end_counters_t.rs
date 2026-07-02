#![allow(non_camel_case_types, non_snake_case)]
use core::ffi::c_int;

/// Raven `backEndCounters_t` — per-frame backend rendering statistics.
///
/// Type definition source: `oracle/oracle/code/renderer/tr_local.h:1075-1087`
#[repr(C)]
pub struct backEndCounters_t {
    pub c_surfaces: c_int,
    pub c_shaders: c_int,
    pub c_vertexes: c_int,
    pub c_indexes: c_int,
    pub c_totalIndexes: c_int,
    pub c_overDraw: f32,

    pub c_dlightVertexes: c_int,
    pub c_dlightIndexes: c_int,

    pub c_flareAdds: c_int,
    pub c_flareTests: c_int,
    pub c_flareRenders: c_int,

    // total msec for backend run
    pub msec: c_int,
}

const _: () = assert!(core::mem::size_of::<backEndCounters_t>() == 48);
const _: () = assert!(core::mem::offset_of!(backEndCounters_t, c_surfaces) == 0);
const _: () = assert!(core::mem::offset_of!(backEndCounters_t, c_shaders) == 4);
const _: () = assert!(core::mem::offset_of!(backEndCounters_t, c_vertexes) == 8);
const _: () = assert!(core::mem::offset_of!(backEndCounters_t, c_indexes) == 12);
const _: () = assert!(core::mem::offset_of!(backEndCounters_t, c_totalIndexes) == 16);
const _: () = assert!(core::mem::offset_of!(backEndCounters_t, c_overDraw) == 20);
const _: () = assert!(core::mem::offset_of!(backEndCounters_t, c_dlightVertexes) == 24);
const _: () = assert!(core::mem::offset_of!(backEndCounters_t, c_dlightIndexes) == 28);
const _: () = assert!(core::mem::offset_of!(backEndCounters_t, c_flareAdds) == 32);
const _: () = assert!(core::mem::offset_of!(backEndCounters_t, c_flareTests) == 36);
const _: () = assert!(core::mem::offset_of!(backEndCounters_t, c_flareRenders) == 40);
const _: () = assert!(core::mem::offset_of!(backEndCounters_t, msec) == 44);
