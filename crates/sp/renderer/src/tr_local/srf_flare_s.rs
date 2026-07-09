#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::shared::vec3_t;

use super::surface_type_t::surfaceType_t;

/// Raven `srfFlare_t` — a lens-flare surface.
///
/// Type definition source: `oracle/code/renderer/tr_local.h:643-648`
#[repr(C)]
pub struct srfFlare_s {
    pub surfaceType: surfaceType_t,
    pub origin: vec3_t,
    pub normal: vec3_t,
    pub color: vec3_t,
}

pub type srfFlare_t = srfFlare_s;

const _: () = assert!(core::mem::size_of::<srfFlare_t>() == 40);
const _: () = assert!(core::mem::offset_of!(srfFlare_t, surfaceType) == 0);
const _: () = assert!(core::mem::offset_of!(srfFlare_t, origin) == 4);
const _: () = assert!(core::mem::offset_of!(srfFlare_t, normal) == 16);
const _: () = assert!(core::mem::offset_of!(srfFlare_t, color) == 28);
