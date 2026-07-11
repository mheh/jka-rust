#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::common::mp::cgame::glconfig_t::glconfig_t;
use mp_qshared::shared::{qboolean, qhandle_t};

/// Raven `uiStatic_t` — per-frame UI module state.
///
/// Type definition source: `oracle/codemp/ui/ui_local.h:538-557`
#[repr(C)]
pub struct uiStatic_t {
    pub frametime: i32,
    pub realtime: i32,
    pub cursorx: i32,
    pub cursory: i32,
    pub glconfig: glconfig_t,
    pub debug: qboolean,
    pub whiteShader: qhandle_t,
    pub menuBackShader: qhandle_t,
    pub menuBackShader2: qhandle_t,
    pub menuBackNoLogoShader: qhandle_t,
    pub charset: qhandle_t,
    pub cursor: qhandle_t,
    pub rb_on: qhandle_t,
    pub rb_off: qhandle_t,
    pub scale: f32,
    pub bias: f32,
    pub demoversion: qboolean,
    pub firstdraw: qboolean,
}

const _: () = assert!(core::mem::size_of::<uiStatic_t>() == 168);
const _: () = assert!(core::mem::offset_of!(uiStatic_t, frametime) == 0);
const _: () = assert!(core::mem::offset_of!(uiStatic_t, realtime) == 4);
const _: () = assert!(core::mem::offset_of!(uiStatic_t, cursorx) == 8);
const _: () = assert!(core::mem::offset_of!(uiStatic_t, cursory) == 12);
const _: () = assert!(core::mem::offset_of!(uiStatic_t, glconfig) == 16);
const _: () = assert!(core::mem::offset_of!(uiStatic_t, debug) == 112);
const _: () = assert!(core::mem::offset_of!(uiStatic_t, whiteShader) == 116);
const _: () = assert!(core::mem::offset_of!(uiStatic_t, menuBackShader) == 120);
const _: () = assert!(core::mem::offset_of!(uiStatic_t, menuBackShader2) == 124);
const _: () = assert!(core::mem::offset_of!(uiStatic_t, menuBackNoLogoShader) == 128);
const _: () = assert!(core::mem::offset_of!(uiStatic_t, charset) == 132);
const _: () = assert!(core::mem::offset_of!(uiStatic_t, cursor) == 136);
const _: () = assert!(core::mem::offset_of!(uiStatic_t, rb_on) == 140);
const _: () = assert!(core::mem::offset_of!(uiStatic_t, rb_off) == 144);
const _: () = assert!(core::mem::offset_of!(uiStatic_t, scale) == 148);
const _: () = assert!(core::mem::offset_of!(uiStatic_t, bias) == 152);
const _: () = assert!(core::mem::offset_of!(uiStatic_t, demoversion) == 156);
const _: () = assert!(core::mem::offset_of!(uiStatic_t, firstdraw) == 160);
