#![allow(non_camel_case_types, non_snake_case)]

/// Raven `dplane_t` — BSP plane.
///
/// Type definition source: `oracle/codemp/qcommon/../qcommon/qfiles.h:455-458`
#[repr(C)]
pub struct dplane_t {
    pub normal: [f32; 3],
    pub dist: f32,
}

const _: () = assert!(core::mem::size_of::<dplane_t>() == 16);
const _: () = assert!(core::mem::offset_of!(dplane_t, normal) == 0);
const _: () = assert!(core::mem::offset_of!(dplane_t, dist) == 12);
