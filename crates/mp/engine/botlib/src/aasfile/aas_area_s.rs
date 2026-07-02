#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::vec3_t;

/// Raven `aas_area_t` — an AAS area.
///
/// Type definition source: `oracle/oracle/codemp/botlib/aasfile.h:188-197`
#[repr(C)]
pub struct aas_area_t {
	/// number of this area
	pub areanum: i32,
	//3d definition
	/// number of faces used for the boundary of the area
	pub numfaces: i32,
	/// first face in the face index used for the boundary of the area
	pub firstface: i32,
	/// mins of the area
	pub mins: vec3_t,
	/// maxs of the area
	pub maxs: vec3_t,
	/// 'center' of the area
	pub center: vec3_t,
}

pub type aas_area_s = aas_area_t;

const _: () = assert!(core::mem::size_of::<aas_area_t>() == 48);
const _: () = assert!(core::mem::offset_of!(aas_area_t, areanum) == 0);
const _: () = assert!(core::mem::offset_of!(aas_area_t, numfaces) == 4);
const _: () = assert!(core::mem::offset_of!(aas_area_t, firstface) == 8);
const _: () = assert!(core::mem::offset_of!(aas_area_t, mins) == 12);
const _: () = assert!(core::mem::offset_of!(aas_area_t, maxs) == 24);
const _: () = assert!(core::mem::offset_of!(aas_area_t, center) == 36);
