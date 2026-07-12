#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use mp_qshared::common::mp::cgame::poly_s::poly_t;
use mp_qshared::common::mp::cgame::poly_vert_t::polyVert_t;
use mp_qshared::shared::qhandle_t;

/// Number of vertices a decal polygon may hold.
///
/// Source: `oracle/codemp/renderer/tr_local.h:2316`
pub const MAX_VERTS_ON_DECAL_POLY: usize = 10;

/// Raven `decalPoly_s` — a persistent decal polygon queued for rendering.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:2319-2328`
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct decalPoly_t {
    pub time: c_int,
    pub fadetime: c_int,
    pub shader: qhandle_t,
    pub color: [f32; 4],
    pub poly: poly_t,
    pub verts: [polyVert_t; MAX_VERTS_ON_DECAL_POLY],
}

pub type decalPoly_s = decalPoly_t;

const _: () = assert!(core::mem::offset_of!(decalPoly_t, time) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<decalPoly_t>() == 288);
    assert!(core::mem::offset_of!(decalPoly_t, fadetime) == 4);
    assert!(core::mem::offset_of!(decalPoly_t, shader) == 8);
    assert!(core::mem::offset_of!(decalPoly_t, color) == 12);
    assert!(core::mem::offset_of!(decalPoly_t, poly) == 32);
    assert!(core::mem::offset_of!(decalPoly_t, verts) == 48);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<decalPoly_t>() == 280);
    assert!(core::mem::offset_of!(decalPoly_t, fadetime) == 4);
    assert!(core::mem::offset_of!(decalPoly_t, shader) == 8);
    assert!(core::mem::offset_of!(decalPoly_t, color) == 12);
    assert!(core::mem::offset_of!(decalPoly_t, poly) == 28);
    assert!(core::mem::offset_of!(decalPoly_t, verts) == 40);
};
