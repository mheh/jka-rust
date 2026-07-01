#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_float, c_int};

use mp_qshared::shared::{qhandle_t, vec4_t};

use super::rect_def_t::rectDef_t;

/// Raven `windowDef_t` — base UI window definition shared by menus and items.
///
/// Type definition source: `oracle/oracle/codemp/ui/ui_shared.h:122-144`
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct windowDef_t {
    /// client coord rectangle
    pub rect: rectDef_t,
    /// screen coord rectangle
    pub rectClient: rectDef_t,
    pub name: *const c_char,
    /// if it belongs to a group
    pub group: *const c_char,
    /// cinematic name
    pub cinematicName: *const c_char,
    /// cinematic handle
    pub cinematic: c_int,
    pub style: c_int,
    pub border: c_int,
    /// ownerDraw style
    pub ownerDraw: c_int,
    /// show flags for ownerdraw items
    pub ownerDrawFlags: c_int,
    pub borderSize: c_float,
    /// visible, focus, mouseover, cursor
    pub flags: c_int,
    /// for various effects
    pub rectEffects: rectDef_t,
    /// for various effects
    pub rectEffects2: rectDef_t,
    /// time based value for various effects
    pub offsetTime: c_int,
    /// time next effect should cycle
    pub nextTime: c_int,
    /// text color
    pub foreColor: vec4_t,
    /// border color
    pub backColor: vec4_t,
    /// border color
    pub borderColor: vec4_t,
    /// border color
    pub outlineColor: vec4_t,
    /// background asset
    pub background: qhandle_t,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<windowDef_t>() == 192);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(windowDef_t, rect) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(windowDef_t, rectClient) == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(windowDef_t, name) == 32);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(windowDef_t, group) == 40);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(windowDef_t, cinematicName) == 48);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(windowDef_t, cinematic) == 56);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(windowDef_t, style) == 60);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(windowDef_t, border) == 64);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(windowDef_t, ownerDraw) == 68);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(windowDef_t, ownerDrawFlags) == 72);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(windowDef_t, borderSize) == 76);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(windowDef_t, flags) == 80);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(windowDef_t, rectEffects) == 84);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(windowDef_t, rectEffects2) == 100);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(windowDef_t, offsetTime) == 116);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(windowDef_t, nextTime) == 120);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(windowDef_t, foreColor) == 124);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(windowDef_t, backColor) == 140);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(windowDef_t, borderColor) == 156);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(windowDef_t, outlineColor) == 172);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(windowDef_t, background) == 188);
