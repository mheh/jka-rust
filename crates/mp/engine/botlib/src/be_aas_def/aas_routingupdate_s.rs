#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_ushort;

use mp_qshared::shared::{qboolean, vec3_t};

/// Raven `aas_routingupdate_t` — a pending routing-cache update, linked into
/// the update list while queued.
///
/// Type definition source: `oracle/oracle/codemp/botlib/be_aas_def.h:150-160`
#[repr(C)]
pub struct aas_routingupdate_t {
	pub cluster: i32,
	pub areanum: i32,               // area number of the update
	pub start: vec3_t,               // start point the area was entered
	pub tmptraveltime: c_ushort,     // temporary travel time
	pub areatraveltimes: *mut c_ushort, // travel times within the area
	pub inlist: qboolean,            // true if the update is in the list
	pub next: *mut aas_routingupdate_t,
	pub prev: *mut aas_routingupdate_t,
}

/// Raven's C tag is `aas_routingupdate_s`; the typedef name
/// `aas_routingupdate_t` is house style for the struct itself.
pub type aas_routingupdate_s = aas_routingupdate_t;

const _: () = assert!(core::mem::size_of::<aas_routingupdate_t>() == 56);
const _: () = assert!(core::mem::offset_of!(aas_routingupdate_t, cluster) == 0);
const _: () = assert!(core::mem::offset_of!(aas_routingupdate_t, areanum) == 4);
const _: () = assert!(core::mem::offset_of!(aas_routingupdate_t, start) == 8);
const _: () = assert!(core::mem::offset_of!(aas_routingupdate_t, tmptraveltime) == 20);
const _: () = assert!(core::mem::offset_of!(aas_routingupdate_t, areatraveltimes) == 24);
const _: () = assert!(core::mem::offset_of!(aas_routingupdate_t, inlist) == 32);
const _: () = assert!(core::mem::offset_of!(aas_routingupdate_t, next) == 40);
const _: () = assert!(core::mem::offset_of!(aas_routingupdate_t, prev) == 48);
