#![allow(non_camel_case_types, non_snake_case)]

/// Raven `aas_edge_s` — an AAS edge, referencing its two endpoint vertexes.
///
/// Raven: none.
/// Type definition source: `oracle/codemp/botlib/aasfile.h:165-168`
#[repr(C)]
pub struct aas_edge_t {
    /// numbers of the vertexes of this edge
    pub v: [i32; 2],
}

pub type aas_edge_s = aas_edge_t;

const _: () = assert!(core::mem::size_of::<aas_edge_t>() == 8);
const _: () = assert!(core::mem::offset_of!(aas_edge_t, v) == 0);
