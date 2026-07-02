#![allow(non_camel_case_types, non_snake_case)]

use crate::shared::vec3_t;

use super::aas_trace_s::aas_trace_s;

/// Raven `aas_clientmove_t` — result of client movement prediction.
///
/// Type definition source: `oracle/oracle/codemp/game/be_aas.h:162-173`
#[repr(C)]
pub struct aas_clientmove_t {
	/// position at the end of movement prediction
	pub endpos: vec3_t,
	/// area at end of movement prediction
	pub endarea: i32,
	/// velocity at the end of movement prediction
	pub velocity: vec3_t,
	/// last trace
	pub trace: aas_trace_s,
	/// presence type at end of movement prediction
	pub presencetype: i32,
	/// event that made the prediction stop
	pub stopevent: i32,
	/// contents at the end of movement prediction
	pub endcontents: i32,
	/// time predicted ahead
	pub time: f32,
	/// number of frames predicted ahead
	pub frames: i32,
}

pub type aas_clientmove_s = aas_clientmove_t;

const _: () = assert!(core::mem::size_of::<aas_clientmove_t>() == 84);
const _: () = assert!(core::mem::offset_of!(aas_clientmove_t, endpos) == 0);
const _: () = assert!(core::mem::offset_of!(aas_clientmove_t, endarea) == 12);
const _: () = assert!(core::mem::offset_of!(aas_clientmove_t, velocity) == 16);
const _: () = assert!(core::mem::offset_of!(aas_clientmove_t, trace) == 28);
const _: () = assert!(core::mem::offset_of!(aas_clientmove_t, presencetype) == 64);
const _: () = assert!(core::mem::offset_of!(aas_clientmove_t, stopevent) == 68);
const _: () = assert!(core::mem::offset_of!(aas_clientmove_t, endcontents) == 72);
const _: () = assert!(core::mem::offset_of!(aas_clientmove_t, time) == 76);
const _: () = assert!(core::mem::offset_of!(aas_clientmove_t, frames) == 80);
