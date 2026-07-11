#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::vec3_t;

/// Raven `ragCallbackDebugBox_t` — ragdoll debug-box callback payload.
///
/// Type definition source: `oracle/codemp/cgame/cg_public.h:542-547`
#[repr(C)]
pub struct ragCallbackDebugBox_t {
    pub mins: vec3_t,
    pub maxs: vec3_t,
    pub duration: i32,
}

const _: () = assert!(core::mem::size_of::<ragCallbackDebugBox_t>() == 28);
const _: () = assert!(core::mem::offset_of!(ragCallbackDebugBox_t, mins) == 0);
const _: () = assert!(core::mem::offset_of!(ragCallbackDebugBox_t, maxs) == 12);
const _: () = assert!(core::mem::offset_of!(ragCallbackDebugBox_t, duration) == 24);
