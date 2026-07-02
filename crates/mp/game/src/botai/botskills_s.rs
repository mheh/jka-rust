#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

/// Raven `botskills_t` — per-bot difficulty/skill tuning values.
///
/// Type definition source: `oracle/oracle/codemp/game/ai_main.h:137-145`
#[repr(C)]
pub struct botskills_t {
	pub reflex: c_int,
	pub accuracy: f32,
	pub turnspeed: f32,
	pub turnspeed_combat: f32,
	pub maxturn: f32,
	pub perfectaim: c_int,
}

const _: () = assert!(core::mem::size_of::<botskills_t>() == 24);
const _: () = assert!(core::mem::offset_of!(botskills_t, reflex) == 0);
const _: () = assert!(core::mem::offset_of!(botskills_t, accuracy) == 4);
const _: () = assert!(core::mem::offset_of!(botskills_t, turnspeed) == 8);
const _: () = assert!(core::mem::offset_of!(botskills_t, turnspeed_combat) == 12);
const _: () = assert!(core::mem::offset_of!(botskills_t, maxturn) == 16);
const _: () = assert!(core::mem::offset_of!(botskills_t, perfectaim) == 20);
