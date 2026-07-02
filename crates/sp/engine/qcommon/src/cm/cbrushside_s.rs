#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;
use sp_qshared::shared::collision::cplane_t;

/// Raven `cbrushside_t` — one side (plane + shader) of a collision-model brush.
///
/// Type definition source: `oracle/oracle/code/qcommon/cm_local.h:57-60`
#[repr(C)]
pub struct cbrushside_s {
    pub plane: *mut cplane_t,
    pub shaderNum: c_int,
}

pub type cbrushside_t = cbrushside_s;

const _: () = assert!(core::mem::size_of::<cbrushside_t>() == 16);
const _: () = assert!(core::mem::offset_of!(cbrushside_t, plane) == 0);
const _: () = assert!(core::mem::offset_of!(cbrushside_t, shaderNum) == 8);
