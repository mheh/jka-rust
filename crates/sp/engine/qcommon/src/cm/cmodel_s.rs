#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::shared::vec3_t;

use super::c_leaf_t::cLeaf_t;

/// Raven `cmodel_t` — a collision model (a submodel's bounds + leaf; submodels don't
/// reference the main tree).
///
/// Type definition source: `oracle/oracle/code/qcommon/cm_local.h:42-45`
#[repr(C)]
pub struct cmodel_s {
    pub mins: vec3_t,
    pub maxs: vec3_t,
    pub leaf: cLeaf_t, // submodels don't reference the main tree
}

pub type cmodel_t = cmodel_s;

const _: () = assert!(core::mem::size_of::<cmodel_t>() == 48);
const _: () = assert!(core::mem::offset_of!(cmodel_t, mins) == 0);
const _: () = assert!(core::mem::offset_of!(cmodel_t, maxs) == 12);
const _: () = assert!(core::mem::offset_of!(cmodel_t, leaf) == 24);
