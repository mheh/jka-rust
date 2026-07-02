#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use sp_qshared::shared::{qboolean, vec3_t};

/// `CM_MAX_GRID_SIZE`.
///
/// Raven: max dimensions of a grid mesh in memory (collision-model variant).
/// Source: `oracle/oracle/code/qcommon/cm_patch.h:104`
pub const CM_MAX_GRID_SIZE: usize = 129;

/// Raven `cGrid_t`.
///
/// Type definition source: `oracle/oracle/code/qcommon/cm_patch.h:104-110`
#[repr(C)]
pub struct cGrid_t {
    pub width: c_int,
    pub height: c_int,
    pub wrapWidth: qboolean,
    pub wrapHeight: qboolean,
    // [width][height]
    pub points: [[vec3_t; CM_MAX_GRID_SIZE]; CM_MAX_GRID_SIZE],
}

const _: () = assert!(core::mem::size_of::<cGrid_t>() == 199708);
const _: () = assert!(core::mem::offset_of!(cGrid_t, width) == 0);
const _: () = assert!(core::mem::offset_of!(cGrid_t, height) == 4);
const _: () = assert!(core::mem::offset_of!(cGrid_t, wrapWidth) == 8);
const _: () = assert!(core::mem::offset_of!(cGrid_t, wrapHeight) == 12);
const _: () = assert!(core::mem::offset_of!(cGrid_t, points) == 16);
