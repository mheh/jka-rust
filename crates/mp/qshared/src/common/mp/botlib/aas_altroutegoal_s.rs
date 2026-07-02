#![allow(non_camel_case_types, non_snake_case)]

use crate::shared::vec3_t;

/// Raven `aas_altroutegoal_t` — alternate route goal for AAS routing.
///
/// Type definition source: `oracle/oracle/codemp/game/be_aas.h:180-187`
#[repr(C)]
pub struct aas_altroutegoal_t {
	pub origin: vec3_t,
	pub areanum: i32,
	pub starttraveltime: u16,
	pub goaltraveltime: u16,
	pub extratraveltime: u16,
}

pub type aas_altroutegoal_s = aas_altroutegoal_t;

const _: () = assert!(core::mem::size_of::<aas_altroutegoal_t>() == 24);
const _: () = assert!(core::mem::offset_of!(aas_altroutegoal_t, origin) == 0);
const _: () = assert!(core::mem::offset_of!(aas_altroutegoal_t, areanum) == 12);
const _: () = assert!(core::mem::offset_of!(aas_altroutegoal_t, starttraveltime) == 16);
const _: () = assert!(core::mem::offset_of!(aas_altroutegoal_t, goaltraveltime) == 18);
const _: () = assert!(core::mem::offset_of!(aas_altroutegoal_t, extratraveltime) == 20);
