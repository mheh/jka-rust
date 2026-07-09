#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;
use mp_qshared::shared::vec3_t;

use super::c_leaf_t::cLeaf_t;

/// Raven `cmodel_t` — a collision model (a submodel's bounds + leaf + main-tree node).
///
/// Type definition source: `oracle/codemp/qcommon/cm_local.h:45-49`
#[repr(C)]
pub struct cmodel_s {
    pub mins: vec3_t,
    pub maxs: vec3_t,
    pub leaf: cLeaf_t, // submodels don't reference the main tree
    pub firstNode: c_int, // only for cmodel[0] (for the main and bsp instances)
}

pub type cmodel_t = cmodel_s;

const _: () = assert!(core::mem::size_of::<cmodel_t>() == 52);
const _: () = assert!(core::mem::offset_of!(cmodel_t, mins) == 0);
const _: () = assert!(core::mem::offset_of!(cmodel_t, maxs) == 12);
const _: () = assert!(core::mem::offset_of!(cmodel_t, leaf) == 24);
const _: () = assert!(core::mem::offset_of!(cmodel_t, firstNode) == 48);
