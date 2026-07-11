#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::vec3_t;

/// Raven `aas_reachability_t` — inter-area reachability link.
///
/// Type definition source: `oracle/codemp/botlib/aasfile.h:107-116`
#[derive(Clone, Copy)]
#[repr(C)]
pub struct aas_reachability_t {
	/// number of the reachable area
	pub areanum: i32,
	/// number of the face towards the other area
	pub facenum: i32,
	/// number of the edge towards the other area
	pub edgenum: i32,
	/// start point of inter area movement
	pub start: vec3_t,
	/// end point of inter area movement
	pub end: vec3_t,
	/// type of travel required to get to the area
	pub traveltype: i32,
	/// travel time of the inter area movement
	pub traveltime: u16,
}

pub type aas_reachability_s = aas_reachability_t;

const _: () = assert!(core::mem::size_of::<aas_reachability_t>() == 44);
const _: () = assert!(core::mem::offset_of!(aas_reachability_t, areanum) == 0);
const _: () = assert!(core::mem::offset_of!(aas_reachability_t, facenum) == 4);
const _: () = assert!(core::mem::offset_of!(aas_reachability_t, edgenum) == 8);
const _: () = assert!(core::mem::offset_of!(aas_reachability_t, start) == 12);
const _: () = assert!(core::mem::offset_of!(aas_reachability_t, end) == 24);
const _: () = assert!(core::mem::offset_of!(aas_reachability_t, traveltype) == 36);
const _: () = assert!(core::mem::offset_of!(aas_reachability_t, traveltime) == 40);
