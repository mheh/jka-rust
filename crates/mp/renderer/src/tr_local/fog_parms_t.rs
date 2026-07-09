#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::vec3_t;

/// Raven `fogParms_t` — fog color/depth parameters.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:454-457`
#[repr(C)]
pub struct fogParms_t {
	pub color: vec3_t,
	pub depthForOpaque: f32,
}

const _: () = assert!(core::mem::size_of::<fogParms_t>() == 16);
const _: () = assert!(core::mem::offset_of!(fogParms_t, color) == 0);
const _: () = assert!(core::mem::offset_of!(fogParms_t, depthForOpaque) == 12);
