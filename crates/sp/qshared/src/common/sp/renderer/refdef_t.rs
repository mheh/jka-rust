//! SP `tr_types.h` render-scene definition.

#![allow(non_camel_case_types, non_snake_case)]

use crate::shared::vec3_t;
use native_types::byte;

/// Raven `MAX_MAP_AREA_BYTES` — bit vector of area visibility.
///
/// Source: `oracle/code/game/q_shared.h:232`
pub const MAX_MAP_AREA_BYTES: usize = 32;

/// Raven `refdef_t` — the scene definition cgame/ui hand to the renderer each frame.
///
/// SP diverges from MP: no `viewangles` field, and the deform-text `text`
/// array is commented out in the SP oracle.
/// Type definition source: `oracle/code/renderer/tr_types.h:159-176`
#[repr(C)]
pub struct refdef_t {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub fov_x: f32,
    pub fov_y: f32,
    pub vieworg: vec3_t,
    pub viewaxis: [vec3_t; 3], // transformation matrix
    pub viewContents: i32,     // world contents at vieworg

    // Raven: time in milliseconds for shader effects and other time dependent rendering issues
    pub time: i32,

    pub rdflags: i32, // RDF_NOWORLDMODEL, etc

    // Raven: 1 bits will prevent the associated area from rendering at all
    pub areamask: [byte; MAX_MAP_AREA_BYTES],
    // Raven (commented out in oracle): text messages for deform text shaders
    //	char		text[MAX_RENDER_STRINGS][MAX_RENDER_STRING_LENGTH];
}

const _: () = assert!(core::mem::size_of::<refdef_t>() == 116);
const _: () = assert!(core::mem::offset_of!(refdef_t, x) == 0);
const _: () = assert!(core::mem::offset_of!(refdef_t, y) == 4);
const _: () = assert!(core::mem::offset_of!(refdef_t, width) == 8);
const _: () = assert!(core::mem::offset_of!(refdef_t, height) == 12);
const _: () = assert!(core::mem::offset_of!(refdef_t, fov_x) == 16);
const _: () = assert!(core::mem::offset_of!(refdef_t, fov_y) == 20);
const _: () = assert!(core::mem::offset_of!(refdef_t, vieworg) == 24);
const _: () = assert!(core::mem::offset_of!(refdef_t, viewaxis) == 36);
const _: () = assert!(core::mem::offset_of!(refdef_t, viewContents) == 72);
const _: () = assert!(core::mem::offset_of!(refdef_t, time) == 76);
const _: () = assert!(core::mem::offset_of!(refdef_t, rdflags) == 80);
const _: () = assert!(core::mem::offset_of!(refdef_t, areamask) == 84);
