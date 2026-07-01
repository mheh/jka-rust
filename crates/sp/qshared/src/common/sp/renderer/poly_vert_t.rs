#![allow(non_camel_case_types, non_snake_case)]

use crate::shared::vec3_t;

/// Raven `polyVert_t` — a single vertex of a dynamically-added polygon.
///
/// Type definition source: `oracle/oracle/code/renderer/tr_types.h:70-74`
#[repr(C)]
pub struct polyVert_t {
    pub xyz: vec3_t,
    pub st: [f32; 2],
    pub modulate: [u8; 4],
}

const _: () = assert!(core::mem::size_of::<polyVert_t>() == 24);
const _: () = assert!(core::mem::offset_of!(polyVert_t, xyz) == 0);
const _: () = assert!(core::mem::offset_of!(polyVert_t, st) == 12);
const _: () = assert!(core::mem::offset_of!(polyVert_t, modulate) == 20);
