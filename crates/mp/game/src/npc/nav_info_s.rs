#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::common::mp::gentity::gentity_t;
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::vec3_t;

/// Raven `navInfo_t` — navigation query result/state for NPC path movement.
///
/// Type definition source: `oracle/oracle/codemp/game/b_local.h:314-322`
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct navInfo_t {
	pub blocker: *mut gentity_t,
	pub direction: vec3_t,
	pub pathDirection: vec3_t,
	pub distance: f32,
	pub trace: trace_t,
	pub flags: i32,
}

const _: () = assert!(core::mem::size_of::<navInfo_t>() == 88);
const _: () = assert!(core::mem::offset_of!(navInfo_t, blocker) == 0);
const _: () = assert!(core::mem::offset_of!(navInfo_t, direction) == 8);
const _: () = assert!(core::mem::offset_of!(navInfo_t, pathDirection) == 20);
const _: () = assert!(core::mem::offset_of!(navInfo_t, distance) == 32);
const _: () = assert!(core::mem::offset_of!(navInfo_t, trace) == 36);
const _: () = assert!(core::mem::offset_of!(navInfo_t, flags) == 84);
