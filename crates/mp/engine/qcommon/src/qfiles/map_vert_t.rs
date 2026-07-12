#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::vec3_t;

// `MAXLIGHTMAPS` (`qfiles.h:500`) imported from its canonical home on
// `drawVert_t` in the same crate.
use super::draw_vert_t::MAXLIGHTMAPS;

/// Raven `mapVert_t` — BSP-file map vertex record.
///
/// Type definition source: `oracle/codemp/qcommon/qfiles.h:506-512`
#[repr(C)]
pub struct mapVert_t {
    pub xyz: vec3_t,
    pub st: [f32; 2],
    pub lightmap: [[f32; 2]; MAXLIGHTMAPS],
    pub normal: vec3_t,
    pub color: [[u8; 4]; MAXLIGHTMAPS],
}

const _: () = assert!(core::mem::size_of::<mapVert_t>() == 80);
const _: () = assert!(core::mem::offset_of!(mapVert_t, xyz) == 0);
const _: () = assert!(core::mem::offset_of!(mapVert_t, st) == 12);
const _: () = assert!(core::mem::offset_of!(mapVert_t, lightmap) == 20);
const _: () = assert!(core::mem::offset_of!(mapVert_t, normal) == 52);
const _: () = assert!(core::mem::offset_of!(mapVert_t, color) == 64);
