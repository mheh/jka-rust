#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::shared::vec3_t;

/// Light Style Constants.
///
/// Source: `oracle/code/qcommon/qfiles.h:310`
const MAXLIGHTMAPS: usize = 4;

/// Raven `drawVert_t` — BSP surface vertex.
///
/// Type definition source: `oracle/code/qcommon/../qcommon/qfiles.h:524-530`
#[repr(C)]
pub struct drawVert_t {
    pub xyz: vec3_t,
    pub st: [f32; 2],
    pub lightmap: [[f32; 2]; MAXLIGHTMAPS],
    pub normal: vec3_t,
    pub color: [[u8; 4]; MAXLIGHTMAPS],
}

const _: () = assert!(core::mem::size_of::<drawVert_t>() == 80);
const _: () = assert!(core::mem::offset_of!(drawVert_t, xyz) == 0);
const _: () = assert!(core::mem::offset_of!(drawVert_t, st) == 12);
const _: () = assert!(core::mem::offset_of!(drawVert_t, lightmap) == 20);
const _: () = assert!(core::mem::offset_of!(drawVert_t, normal) == 52);
const _: () = assert!(core::mem::offset_of!(drawVert_t, color) == 64);
