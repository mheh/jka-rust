#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use mp_qshared::shared::vec3_t;

/// Raven `nodeobject_t`: a single bot navigation-node.
///
/// Type definition source: `oracle/codemp/game/ai_main.h:115-128`
#[repr(C)]
pub struct nodeobject_t {
    pub origin: vec3_t,
    // int index;
    pub weight: f32,
    pub flags: c_int,
    // Raven gates `neighbornum`/`inuse` to `short` under `_XBOX`.
    // This build targets non-Xbox, so both fields are `int`.
    pub neighbornum: c_int,
    pub inuse: c_int,
}

const _: () = assert!(core::mem::size_of::<nodeobject_t>() == 28);
const _: () = assert!(core::mem::offset_of!(nodeobject_t, origin) == 0);
const _: () = assert!(core::mem::offset_of!(nodeobject_t, weight) == 12);
const _: () = assert!(core::mem::offset_of!(nodeobject_t, flags) == 16);
const _: () = assert!(core::mem::offset_of!(nodeobject_t, neighbornum) == 20);
const _: () = assert!(core::mem::offset_of!(nodeobject_t, inuse) == 24);

// This is a `#[repr(C)]` POD (`vec3_t`, `f32`, `c_int`), so the all-zero image is a valid inhabitant.
// The ~448 KB `nodetable` box builds heap-first through `zeroed_box`.
unsafe impl native_platform::ZeroValid for nodeobject_t {}
