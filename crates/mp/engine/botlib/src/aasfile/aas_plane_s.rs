#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::vec3_t;

/// Raven `aas_plane_t` — AAS map plane.
///
/// Type definition source: `oracle/oracle/codemp/botlib/aasfile.h:157-162`
#[repr(C)]
pub struct aas_plane_t {
	/// normal vector of the plane
	pub normal: vec3_t,
	/// distance of the plane (normal vector * distance = point in plane)
	pub dist: f32,
	pub r#type: i32,
}

pub type aas_plane_s = aas_plane_t;

const _: () = assert!(core::mem::size_of::<aas_plane_t>() == 20);
const _: () = assert!(core::mem::offset_of!(aas_plane_t, normal) == 0);
const _: () = assert!(core::mem::offset_of!(aas_plane_t, dist) == 12);
const _: () = assert!(core::mem::offset_of!(aas_plane_t, r#type) == 16);
