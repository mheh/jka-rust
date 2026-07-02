#![allow(non_camel_case_types, non_snake_case)]

use super::surface_type_t::surfaceType_t;

/// Raven `srfDisplayList_t` — a compiled OpenGL display list surface.
///
/// Type definition source: `oracle/oracle/code/renderer/tr_local.h:628-631`
#[repr(C)]
pub struct srfDisplayList_s {
    pub surfaceType: surfaceType_t,
    pub listNum: i32,
}

pub type srfDisplayList_t = srfDisplayList_s;

const _: () = assert!(core::mem::size_of::<srfDisplayList_t>() == 8);
const _: () = assert!(core::mem::offset_of!(srfDisplayList_t, surfaceType) == 0);
const _: () = assert!(core::mem::offset_of!(srfDisplayList_t, listNum) == 4);
