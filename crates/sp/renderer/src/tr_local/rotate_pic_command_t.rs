#![allow(non_camel_case_types, non_snake_case)]

use super::shader_s::shader_s;

/// Raven `rotatePicCommand_t` — render-command to draw a rotated pic.
///
/// Type definition source: `oracle/oracle/code/renderer/tr_local.h:2018-2026`
#[repr(C)]
pub struct rotatePicCommand_t {
    pub commandId: i32,
    pub shader: *mut shader_s,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub s1: f32,
    pub t1: f32,
    pub s2: f32,
    pub t2: f32,
    pub a: f32,
}

const _: () = assert!(core::mem::size_of::<rotatePicCommand_t>() == 56);
const _: () = assert!(core::mem::offset_of!(rotatePicCommand_t, commandId) == 0);
const _: () = assert!(core::mem::offset_of!(rotatePicCommand_t, shader) == 8);
const _: () = assert!(core::mem::offset_of!(rotatePicCommand_t, x) == 16);
const _: () = assert!(core::mem::offset_of!(rotatePicCommand_t, y) == 20);
const _: () = assert!(core::mem::offset_of!(rotatePicCommand_t, w) == 24);
const _: () = assert!(core::mem::offset_of!(rotatePicCommand_t, h) == 28);
const _: () = assert!(core::mem::offset_of!(rotatePicCommand_t, s1) == 32);
const _: () = assert!(core::mem::offset_of!(rotatePicCommand_t, t1) == 36);
const _: () = assert!(core::mem::offset_of!(rotatePicCommand_t, s2) == 40);
const _: () = assert!(core::mem::offset_of!(rotatePicCommand_t, t2) == 44);
const _: () = assert!(core::mem::offset_of!(rotatePicCommand_t, a) == 48);
