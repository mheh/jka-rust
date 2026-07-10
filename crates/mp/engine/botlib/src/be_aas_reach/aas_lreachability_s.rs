#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::vec3_t;

/// Raven `aas_lreachability_t` — a temporary (loading) reachability link.
///
/// Type definition source: `oracle/codemp/botlib/be_aas_reach.cpp:70-81`
#[repr(C)]
pub struct aas_lreachability_t {
	pub areanum: i32,
	pub facenum: i32,
	pub edgenum: i32,
	pub start: vec3_t,
	pub end: vec3_t,
	pub traveltype: i32,
	pub traveltime: u16,
	pub next: *mut aas_lreachability_t,
}

pub type aas_lreachability_s = aas_lreachability_t;

const _: () = assert!(core::mem::size_of::<aas_lreachability_t>() == 56);
const _: () = assert!(core::mem::offset_of!(aas_lreachability_t, areanum) == 0);
const _: () = assert!(core::mem::offset_of!(aas_lreachability_t, facenum) == 4);
const _: () = assert!(core::mem::offset_of!(aas_lreachability_t, edgenum) == 8);
const _: () = assert!(core::mem::offset_of!(aas_lreachability_t, start) == 12);
const _: () = assert!(core::mem::offset_of!(aas_lreachability_t, end) == 24);
const _: () = assert!(core::mem::offset_of!(aas_lreachability_t, traveltype) == 36);
const _: () = assert!(core::mem::offset_of!(aas_lreachability_t, traveltime) == 40);
const _: () = assert!(core::mem::offset_of!(aas_lreachability_t, next) == 48);
