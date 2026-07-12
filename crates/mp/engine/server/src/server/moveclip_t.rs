#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::vec3_t;

/// Raven `moveclip_t` — the working state of a single `SV_Trace` sweep.
///
/// Type definition source: `oracle/codemp/server/sv_world.cpp:440-461`
#[repr(C)]
pub struct moveclip_t {
    /// enclose the test object along entire move
    pub boxmins: vec3_t,
    pub boxmaxs: vec3_t,
    pub mins: *const f32,
    /// size of the moving object
    pub maxs: *const f32,
    pub start: vec3_t,
    pub end: vec3_t,
    pub passEntityNum: i32,
    pub contentmask: i32,
    pub capsule: i32,
    pub traceFlags: i32,
    pub useLod: i32,
    /// make sure nothing goes under here for Ghoul2 collision purposes
    pub trace: trace_t,
}

const _: () = assert!(core::mem::size_of::<moveclip_t>() == 136);
const _: () = assert!(core::mem::offset_of!(moveclip_t, boxmins) == 0);
const _: () = assert!(core::mem::offset_of!(moveclip_t, boxmaxs) == 12);
const _: () = assert!(core::mem::offset_of!(moveclip_t, mins) == 24);
const _: () = assert!(core::mem::offset_of!(moveclip_t, maxs) == 32);
const _: () = assert!(core::mem::offset_of!(moveclip_t, start) == 40);
const _: () = assert!(core::mem::offset_of!(moveclip_t, end) == 52);
const _: () = assert!(core::mem::offset_of!(moveclip_t, passEntityNum) == 64);
const _: () = assert!(core::mem::offset_of!(moveclip_t, contentmask) == 68);
const _: () = assert!(core::mem::offset_of!(moveclip_t, capsule) == 72);
const _: () = assert!(core::mem::offset_of!(moveclip_t, traceFlags) == 76);
const _: () = assert!(core::mem::offset_of!(moveclip_t, useLod) == 80);
const _: () = assert!(core::mem::offset_of!(moveclip_t, trace) == 84);
