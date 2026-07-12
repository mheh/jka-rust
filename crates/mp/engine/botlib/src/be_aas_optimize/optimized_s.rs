#![allow(non_camel_case_types, non_snake_case)]

use crate::aasfile::aas_area_s::aas_area_t;
use crate::aasfile::aas_edge_s::aas_edge_t;
use crate::aasfile::aas_edgeindex_t::aas_edgeindex_t;
use crate::aasfile::aas_face_s::aas_face_t;
use crate::aasfile::aas_faceindex_t::aas_faceindex_t;
use crate::aasfile::aas_vertex_t::aas_vertex_t;

/// Raven `optimized_t` — scratch buffers for AAS file optimization.
///
/// Type definition source: `oracle/codemp/botlib/be_aas_optimize.cpp:29-53`
#[repr(C)]
pub struct optimized_t {
    pub numvertexes: i32,
    pub vertexes: *mut aas_vertex_t,
    pub numedges: i32,
    pub edges: *mut aas_edge_t,
    pub edgeindexsize: i32,
    pub edgeindex: *mut aas_edgeindex_t,
    pub numfaces: i32,
    pub faces: *mut aas_face_t,
    pub faceindexsize: i32,
    pub faceindex: *mut aas_faceindex_t,
    pub numareas: i32,
    pub areas: *mut aas_area_t,
    pub vertexoptimizeindex: *mut i32,
    pub edgeoptimizeindex: *mut i32,
    pub faceoptimizeindex: *mut i32,
}

pub type optimized_s = optimized_t;

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<optimized_t>() == 120);
    assert!(core::mem::offset_of!(optimized_t, numvertexes) == 0);
    assert!(core::mem::offset_of!(optimized_t, vertexes) == 8);
    assert!(core::mem::offset_of!(optimized_t, numedges) == 16);
    assert!(core::mem::offset_of!(optimized_t, edges) == 24);
    assert!(core::mem::offset_of!(optimized_t, edgeindexsize) == 32);
    assert!(core::mem::offset_of!(optimized_t, edgeindex) == 40);
    assert!(core::mem::offset_of!(optimized_t, numfaces) == 48);
    assert!(core::mem::offset_of!(optimized_t, faces) == 56);
    assert!(core::mem::offset_of!(optimized_t, faceindexsize) == 64);
    assert!(core::mem::offset_of!(optimized_t, faceindex) == 72);
    assert!(core::mem::offset_of!(optimized_t, numareas) == 80);
    assert!(core::mem::offset_of!(optimized_t, areas) == 88);
    assert!(core::mem::offset_of!(optimized_t, vertexoptimizeindex) == 96);
    assert!(core::mem::offset_of!(optimized_t, edgeoptimizeindex) == 104);
    assert!(core::mem::offset_of!(optimized_t, faceoptimizeindex) == 112);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<optimized_t>() == 60);
    assert!(core::mem::offset_of!(optimized_t, numvertexes) == 0);
    assert!(core::mem::offset_of!(optimized_t, vertexes) == 4);
    assert!(core::mem::offset_of!(optimized_t, numedges) == 8);
    assert!(core::mem::offset_of!(optimized_t, edges) == 12);
    assert!(core::mem::offset_of!(optimized_t, edgeindexsize) == 16);
    assert!(core::mem::offset_of!(optimized_t, edgeindex) == 20);
    assert!(core::mem::offset_of!(optimized_t, numfaces) == 24);
    assert!(core::mem::offset_of!(optimized_t, faces) == 28);
    assert!(core::mem::offset_of!(optimized_t, faceindexsize) == 32);
    assert!(core::mem::offset_of!(optimized_t, faceindex) == 36);
    assert!(core::mem::offset_of!(optimized_t, numareas) == 40);
    assert!(core::mem::offset_of!(optimized_t, areas) == 44);
    assert!(core::mem::offset_of!(optimized_t, vertexoptimizeindex) == 48);
    assert!(core::mem::offset_of!(optimized_t, edgeoptimizeindex) == 52);
    assert!(core::mem::offset_of!(optimized_t, faceoptimizeindex) == 56);
};
