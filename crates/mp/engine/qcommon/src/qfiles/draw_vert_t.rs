#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::vec3_t;

/// Number of lightmaps per `drawVert_t`.
///
/// Source: `oracle/codemp/qcommon/../qcommon/qfiles.h:500`
pub const MAXLIGHTMAPS: usize = 4;

/// Raven `drawVert_t` — BSP surface vertex.
///
/// Type definition source: `oracle/codemp/qcommon/../qcommon/qfiles.h:514-520`
// `Clone, Copy` added by DEC-43.4 (the `WorldAsset::surfaces` carrier): every
// field is already a plain value array, so the derives are layout-neutral (the
// asserts below are unchanged) and let the owned surface payloads that hold
// `Vec<drawVert_t>` — `GridMesh`, `SurfaceTriangles`, hence `Surface` and
// `WorldAsset` — satisfy `RenderAssets`' `Clone` bound (`Arc::make_mut`).
#[derive(Clone, Copy)]
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
