#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::shared::cplane_t;

/// Raven `cNode_t` — a BSP tree node: splitting plane and child indices.
///
/// Raven: negative numbers are leafs.
/// Type definition source: `oracle/oracle/code/qcommon/cm_local.h:24-27`
#[repr(C)]
pub struct cNode_t {
    pub plane: *mut cplane_t,
    pub children: [i32; 2],
}

const _: () = assert!(core::mem::size_of::<cNode_t>() == 16);
const _: () = assert!(core::mem::offset_of!(cNode_t, plane) == 0);
const _: () = assert!(core::mem::offset_of!(cNode_t, children) == 8);
