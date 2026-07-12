#![allow(non_camel_case_types, non_snake_case)]

use super::shader_s::shader_s;

/// Raven `stretchPicCommand_t` — render-command to draw a stretched pic.
///
/// Type definition source: `oracle/code/renderer/tr_local.h:2009-2016`
#[repr(C)]
pub struct stretchPicCommand_t {
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
}

const _: () = assert!(core::mem::size_of::<stretchPicCommand_t>() == 48);
const _: () = assert!(core::mem::offset_of!(stretchPicCommand_t, commandId) == 0);
const _: () = assert!(core::mem::offset_of!(stretchPicCommand_t, shader) == 8);
const _: () = assert!(core::mem::offset_of!(stretchPicCommand_t, x) == 16);
const _: () = assert!(core::mem::offset_of!(stretchPicCommand_t, y) == 20);
const _: () = assert!(core::mem::offset_of!(stretchPicCommand_t, w) == 24);
const _: () = assert!(core::mem::offset_of!(stretchPicCommand_t, h) == 28);
const _: () = assert!(core::mem::offset_of!(stretchPicCommand_t, s1) == 32);
const _: () = assert!(core::mem::offset_of!(stretchPicCommand_t, t1) == 36);
const _: () = assert!(core::mem::offset_of!(stretchPicCommand_t, s2) == 40);
const _: () = assert!(core::mem::offset_of!(stretchPicCommand_t, t2) == 44);
