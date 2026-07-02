#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use mp_qshared::shared::collision::cplane_t;

/// Raven `cNode_t` — a BSP node in the collision model tree.
///
/// Type definition source: `oracle/oracle/codemp/qcommon/cm_local.h:27-30`
#[repr(C)]
pub struct cNode_t {
    pub plane: *mut cplane_t,
    // negative numbers are leafs
    pub children: [c_int; 2],
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<cNode_t>() == 16);
const _: () = assert!(core::mem::offset_of!(cNode_t, plane) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cNode_t, children) == 8);
