#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use mp_qshared::shared::MAX_QPATH;

use super::bot_characteristic_s::bot_characteristic_t;

/// Raven `bot_character_t` — a loaded bot character (variable-sized trailing
/// characteristic array, emulated with `[_; 1]`).
///
/// Type definition source: `oracle/codemp/botlib/be_ai_char.cpp:53-58`
#[repr(C)]
pub struct bot_character_t {
	pub filename: [c_char; MAX_QPATH as usize],
	pub skill: f32,
	/// variable sized
	pub c: [bot_characteristic_t; 1],
}

pub type bot_character_s = bot_character_t;

const _: () = assert!(core::mem::size_of::<bot_character_t>() == 88);
const _: () = assert!(core::mem::offset_of!(bot_character_t, filename) == 0);
const _: () = assert!(core::mem::offset_of!(bot_character_t, skill) == 64);
const _: () = assert!(core::mem::offset_of!(bot_character_t, c) == 72);
