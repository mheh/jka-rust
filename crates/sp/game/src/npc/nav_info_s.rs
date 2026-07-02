#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::common::sp::gentity::gentity_t;
use sp_qshared::common::sp::trace_t::trace_t;
use sp_qshared::shared::vec3_t;

/// Raven `navInfo_t` — navigation query result/state for NPC path movement.
///
/// Type definition source: `oracle/oracle/code/game/b_local.h:340-348`
#[repr(C)]
pub struct navInfo_t {
	pub blocker: *mut gentity_t,
	pub direction: vec3_t,
	pub pathDirection: vec3_t,
	pub distance: f32,
	pub trace: trace_t,
	pub flags: i32,
}

const _: () = assert!(core::mem::size_of::<navInfo_t>() == 1120);
const _: () = assert!(core::mem::offset_of!(navInfo_t, blocker) == 0);
const _: () = assert!(core::mem::offset_of!(navInfo_t, direction) == 8);
const _: () = assert!(core::mem::offset_of!(navInfo_t, pathDirection) == 20);
const _: () = assert!(core::mem::offset_of!(navInfo_t, distance) == 32);
const _: () = assert!(core::mem::offset_of!(navInfo_t, trace) == 36);
const _: () = assert!(core::mem::offset_of!(navInfo_t, flags) == 1116);
