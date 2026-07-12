#![allow(non_camel_case_types, non_snake_case)]

/// Raven `aas_node_t` — BSP node used to represent the world for tracing.
///
/// Type definition source: `oracle/codemp/botlib/aasfile.h:200-205`
#[repr(C)]
pub struct aas_node_t {
    pub planenum: i32,
    /// child nodes of this node, or areas as leaves when negative
    /// when a child is zero it's a solid leaf
    pub children: [i32; 2],
}

pub type aas_node_s = aas_node_t;

const _: () = assert!(core::mem::size_of::<aas_node_t>() == 12);
const _: () = assert!(core::mem::offset_of!(aas_node_t, planenum) == 0);
const _: () = assert!(core::mem::offset_of!(aas_node_t, children) == 4);
