#![allow(non_camel_case_types, non_snake_case)]

use crate::shared::vec3_t;

/// Raven `aas_predictroute_t` — result of AAS route prediction.
///
/// Type definition source: `oracle/codemp/game/be_aas.h:196-205`
#[repr(C)]
pub struct aas_predictroute_t {
	/// position at the end of movement prediction
	pub endpos: vec3_t,
	/// area at end of movement prediction
	pub endarea: i32,
	/// event that made the prediction stop
	pub stopevent: i32,
	/// contents at the end of movement prediction
	pub endcontents: i32,
	/// end travel flags
	pub endtravelflags: i32,
	/// number of areas predicted ahead
	pub numareas: i32,
	/// time predicted ahead (in hundreth of a sec)
	pub time: i32,
}

pub type aas_predictroute_s = aas_predictroute_t;

const _: () = assert!(core::mem::size_of::<aas_predictroute_t>() == 36);
const _: () = assert!(core::mem::offset_of!(aas_predictroute_t, endpos) == 0);
const _: () = assert!(core::mem::offset_of!(aas_predictroute_t, endarea) == 12);
const _: () = assert!(core::mem::offset_of!(aas_predictroute_t, stopevent) == 16);
const _: () = assert!(core::mem::offset_of!(aas_predictroute_t, endcontents) == 20);
const _: () = assert!(core::mem::offset_of!(aas_predictroute_t, endtravelflags) == 24);
const _: () = assert!(core::mem::offset_of!(aas_predictroute_t, numareas) == 28);
const _: () = assert!(core::mem::offset_of!(aas_predictroute_t, time) == 32);
