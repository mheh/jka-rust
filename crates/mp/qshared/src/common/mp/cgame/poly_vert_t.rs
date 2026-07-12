//! MP `tr_types.h` polygon vertex.

#![allow(non_camel_case_types, non_snake_case)]

use native_types::byte;

use crate::shared::vec3_t;

/// Raven `polyVert_t`.
///
/// Type definition source: `oracle/codemp/cgame/tr_types.h:71-75`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct polyVert_t {
    pub xyz: vec3_t,
    pub st: [f32; 2],
    pub modulate: [byte; 4],
}

const _: () = assert!(core::mem::size_of::<polyVert_t>() == 24);
const _: () = assert!(core::mem::offset_of!(polyVert_t, xyz) == 0);
const _: () = assert!(core::mem::offset_of!(polyVert_t, st) == 12);
const _: () = assert!(core::mem::offset_of!(polyVert_t, modulate) == 20);
