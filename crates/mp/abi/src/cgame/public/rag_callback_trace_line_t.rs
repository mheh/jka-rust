#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::vec3_t;

/// Raven `ragCallbackTraceLine_t` — ragdoll trace-line callback payload.
///
/// Type definition source: `oracle/codemp/cgame/cg_public.h:582-591`
#[repr(C)]
pub struct ragCallbackTraceLine_t {
    pub tr: trace_t,
    pub start: vec3_t,
    pub end: vec3_t,
    pub mins: vec3_t,
    pub maxs: vec3_t,
    pub ignore: i32,
    pub mask: i32,
}

const _: () = assert!(core::mem::size_of::<ragCallbackTraceLine_t>() == 104);
const _: () = assert!(core::mem::offset_of!(ragCallbackTraceLine_t, tr) == 0);
const _: () = assert!(core::mem::offset_of!(ragCallbackTraceLine_t, start) == 48);
const _: () = assert!(core::mem::offset_of!(ragCallbackTraceLine_t, end) == 60);
const _: () = assert!(core::mem::offset_of!(ragCallbackTraceLine_t, mins) == 72);
const _: () = assert!(core::mem::offset_of!(ragCallbackTraceLine_t, maxs) == 84);
const _: () = assert!(core::mem::offset_of!(ragCallbackTraceLine_t, ignore) == 96);
const _: () = assert!(core::mem::offset_of!(ragCallbackTraceLine_t, mask) == 100);
