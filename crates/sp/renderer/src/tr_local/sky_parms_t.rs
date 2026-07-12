#![allow(non_camel_case_types, non_snake_case)]

use crate::tr_local::image_s::image_t;

/// Raven `skyParms_t`.
///
/// Raven: `//	image_t		*outerbox[6], *innerbox[6];`
/// Type definition source: `oracle/code/renderer/tr_local.h:434-438`
#[repr(C)]
pub struct skyParms_t {
    pub cloudHeight: f32,
    pub outerbox: [*mut image_t; 6],
}

const _: () = assert!(core::mem::size_of::<skyParms_t>() == 56);
const _: () = assert!(core::mem::offset_of!(skyParms_t, cloudHeight) == 0);
const _: () = assert!(core::mem::offset_of!(skyParms_t, outerbox) == 8);
