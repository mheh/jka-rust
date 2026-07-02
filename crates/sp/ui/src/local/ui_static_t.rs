#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::common::sp::renderer::glconfig_t::glconfig_t;
use sp_qshared::shared::{qboolean, qhandle_t};

/// Raven `uiStatic_t` — per-frame UI module state.
///
/// Type definition source: `oracle/oracle/code/ui/ui_local.h:66-81`
#[repr(C)]
pub struct uiStatic_t {
	pub frametime: i32,
	pub realtime: i32,
	pub cursorx: i32,
	pub cursory: i32,

	pub glconfig: glconfig_t,
	pub debugMode: qboolean,
	pub whiteShader: qhandle_t,
	pub menuBackShader: qhandle_t,
	pub cursor: qhandle_t,
	pub scalex: f32,
	pub scaley: f32,
	//float				bias;
	pub firstdraw: qboolean,
}

const _: () = assert!(core::mem::size_of::<uiStatic_t>() == 144);
const _: () = assert!(core::mem::offset_of!(uiStatic_t, frametime) == 0);
const _: () = assert!(core::mem::offset_of!(uiStatic_t, realtime) == 4);
const _: () = assert!(core::mem::offset_of!(uiStatic_t, cursorx) == 8);
const _: () = assert!(core::mem::offset_of!(uiStatic_t, cursory) == 12);
const _: () = assert!(core::mem::offset_of!(uiStatic_t, glconfig) == 16);
const _: () = assert!(core::mem::offset_of!(uiStatic_t, debugMode) == 112);
const _: () = assert!(core::mem::offset_of!(uiStatic_t, whiteShader) == 116);
const _: () = assert!(core::mem::offset_of!(uiStatic_t, menuBackShader) == 120);
const _: () = assert!(core::mem::offset_of!(uiStatic_t, cursor) == 124);
const _: () = assert!(core::mem::offset_of!(uiStatic_t, scalex) == 128);
const _: () = assert!(core::mem::offset_of!(uiStatic_t, scaley) == 132);
const _: () = assert!(core::mem::offset_of!(uiStatic_t, firstdraw) == 136);
