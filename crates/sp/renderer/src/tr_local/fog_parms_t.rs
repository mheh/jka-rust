#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::shared::vec3_t;

/// Raven `fogParms_t` — fog color and opaque depth for a fog volume.
///
/// Type definition source: `oracle/oracle/code/renderer/tr_local.h:440-443`
#[repr(C)]
pub struct fogParms_t {
    pub color: vec3_t,
    pub depthForOpaque: f32,
}

const _: () = assert!(core::mem::size_of::<fogParms_t>() == 16);
const _: () = assert!(core::mem::offset_of!(fogParms_t, color) == 0);
const _: () = assert!(core::mem::offset_of!(fogParms_t, depthForOpaque) == 12);
