#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::shared::vec3_t;

/// Raven `dlight_s` (typedef `dlight_t`) — dynamic light.
///
/// Type definition source: `oracle/oracle/code/renderer/tr_local.h:43-49`
#[repr(C)]
pub struct dlight_t {
    pub origin: vec3_t,
    /// range from 0.0 to 1.0, should be color normalized
    pub color: vec3_t,
    pub radius: f32,

    /// origin in local coordinate system
    pub transformed: vec3_t,
}

/// Raven manifest tag name; the typedef is `dlight_t`.
pub type dlight_s = dlight_t;

const _: () = assert!(core::mem::size_of::<dlight_t>() == 40);
const _: () = assert!(core::mem::offset_of!(dlight_t, origin) == 0);
const _: () = assert!(core::mem::offset_of!(dlight_t, color) == 12);
const _: () = assert!(core::mem::offset_of!(dlight_t, radius) == 24);
const _: () = assert!(core::mem::offset_of!(dlight_t, transformed) == 28);
