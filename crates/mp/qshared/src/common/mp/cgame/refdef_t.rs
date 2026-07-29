//! MP `tr_types.h` render-scene definition.

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use crate::shared::vec3_t;
use native_types::byte;

/// Raven `MAX_MAP_AREA_BYTES` — bit vector of area visibility.
///
/// Source: `oracle/codemp/game/q_shared.h:416`
pub const MAX_MAP_AREA_BYTES: usize = 32;

/// Raven `MAX_RENDER_STRINGS`.
///
/// Source: `oracle/codemp/cgame/tr_types.h:254`
pub const MAX_RENDER_STRINGS: usize = 8;

/// Raven `MAX_RENDER_STRING_LENGTH`.
///
/// Source: `oracle/codemp/cgame/tr_types.h:255`
pub const MAX_RENDER_STRING_LENGTH: usize = 32;

/// Raven `refdef_t` — the scene definition cgame/ui hand to the renderer each frame.
///
/// Type definition source: `oracle/codemp/cgame/tr_types.h:257-275`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct refdef_t {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub fov_x: f32,
    pub fov_y: f32,
    pub vieworg: vec3_t,
    pub viewangles: vec3_t,
    pub viewaxis: [vec3_t; 3], // transformation matrix
    pub viewContents: i32,     // world contents at vieworg

    // Raven: time in milliseconds for shader effects and other time dependent rendering issues
    pub time: i32,

    pub rdflags: i32, // RDF_NOWORLDMODEL, etc

    // Raven: 1 bits will prevent the associated area from rendering at all
    pub areamask: [byte; MAX_MAP_AREA_BYTES],

    // Raven: text messages for deform text shaders
    pub text: [[c_char; MAX_RENDER_STRING_LENGTH]; MAX_RENDER_STRINGS],
}

const _: () = assert!(core::mem::size_of::<refdef_t>() == 384);
const _: () = assert!(core::mem::offset_of!(refdef_t, x) == 0);
const _: () = assert!(core::mem::offset_of!(refdef_t, y) == 4);
const _: () = assert!(core::mem::offset_of!(refdef_t, width) == 8);
const _: () = assert!(core::mem::offset_of!(refdef_t, height) == 12);
const _: () = assert!(core::mem::offset_of!(refdef_t, fov_x) == 16);
const _: () = assert!(core::mem::offset_of!(refdef_t, fov_y) == 20);
const _: () = assert!(core::mem::offset_of!(refdef_t, vieworg) == 24);
const _: () = assert!(core::mem::offset_of!(refdef_t, viewangles) == 36);
const _: () = assert!(core::mem::offset_of!(refdef_t, viewaxis) == 48);
const _: () = assert!(core::mem::offset_of!(refdef_t, viewContents) == 84);
const _: () = assert!(core::mem::offset_of!(refdef_t, time) == 88);
const _: () = assert!(core::mem::offset_of!(refdef_t, rdflags) == 92);
const _: () = assert!(core::mem::offset_of!(refdef_t, areamask) == 96);
const _: () = assert!(core::mem::offset_of!(refdef_t, text) == 128);
