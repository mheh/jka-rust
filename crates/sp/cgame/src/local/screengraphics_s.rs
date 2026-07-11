#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use sp_qshared::shared::qhandle_t;

/// Raven `screengraphics_s` — a scripted screen HUD element (string or graphic).
///
/// Type definition source: `oracle/code/cgame/cg_local.h:521-539`
#[repr(C)]
pub struct screengraphics_s {
    /// STRING or GRAPHIC
    pub r#type: i32,
    /// When it changes
    pub timer: f32,
    /// X position
    pub x: i32,
    /// Y positon
    pub y: i32,
    /// Graphic width
    pub width: i32,
    /// Graphic height
    pub height: i32,
    /// File name of graphic/ text if STRING
    pub file: *mut c_char,
    /// Index to ingame_text[]
    pub ingameEnum: i32,
    /// Handle of graphic if GRAPHIC
    pub graphic: qhandle_t,
    pub min: i32,
    pub max: i32,
    /// Final value
    pub target: i32,
    pub inc: i32,
    pub style: i32,
    /// Normal color
    pub color: i32,
    /// To an address
    pub pointer: *mut core::ffi::c_void,
}

const _: () = assert!(core::mem::size_of::<screengraphics_s>() == 72);
const _: () = assert!(core::mem::offset_of!(screengraphics_s, r#type) == 0);
const _: () = assert!(core::mem::offset_of!(screengraphics_s, timer) == 4);
const _: () = assert!(core::mem::offset_of!(screengraphics_s, x) == 8);
const _: () = assert!(core::mem::offset_of!(screengraphics_s, y) == 12);
const _: () = assert!(core::mem::offset_of!(screengraphics_s, width) == 16);
const _: () = assert!(core::mem::offset_of!(screengraphics_s, height) == 20);
const _: () = assert!(core::mem::offset_of!(screengraphics_s, file) == 24);
const _: () = assert!(core::mem::offset_of!(screengraphics_s, ingameEnum) == 32);
const _: () = assert!(core::mem::offset_of!(screengraphics_s, graphic) == 36);
const _: () = assert!(core::mem::offset_of!(screengraphics_s, min) == 40);
const _: () = assert!(core::mem::offset_of!(screengraphics_s, max) == 44);
const _: () = assert!(core::mem::offset_of!(screengraphics_s, target) == 48);
const _: () = assert!(core::mem::offset_of!(screengraphics_s, inc) == 52);
const _: () = assert!(core::mem::offset_of!(screengraphics_s, style) == 56);
const _: () = assert!(core::mem::offset_of!(screengraphics_s, color) == 60);
const _: () = assert!(core::mem::offset_of!(screengraphics_s, pointer) == 64);
