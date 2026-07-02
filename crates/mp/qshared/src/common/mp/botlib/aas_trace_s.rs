#![allow(non_camel_case_types, non_snake_case)]

use crate::shared::{qboolean, vec3_t};

/// Raven `aas_trace_t` — result of an AAS trace.
///
/// Type definition source: `oracle/oracle/codemp/game/be_aas.h:68-77`
#[repr(C)]
pub struct aas_trace_t {
	pub startsolid: qboolean, // if true, the initial point was in a solid area
	pub fraction: f32,        // time completed, 1.0 = didn't hit anything
	pub endpos: vec3_t,       // final position
	pub ent: i32,             // entity blocking the trace
	pub lastarea: i32,        // last area the trace was in (zero if none)
	pub area: i32,            // area blocking the trace (zero if none)
	pub planenum: i32,        // number of the plane that was hit
}

pub type aas_trace_s = aas_trace_t;

const _: () = assert!(core::mem::size_of::<aas_trace_t>() == 36);
const _: () = assert!(core::mem::offset_of!(aas_trace_t, startsolid) == 0);
const _: () = assert!(core::mem::offset_of!(aas_trace_t, fraction) == 4);
const _: () = assert!(core::mem::offset_of!(aas_trace_t, endpos) == 8);
const _: () = assert!(core::mem::offset_of!(aas_trace_t, ent) == 20);
const _: () = assert!(core::mem::offset_of!(aas_trace_t, lastarea) == 24);
const _: () = assert!(core::mem::offset_of!(aas_trace_t, area) == 28);
const _: () = assert!(core::mem::offset_of!(aas_trace_t, planenum) == 32);
