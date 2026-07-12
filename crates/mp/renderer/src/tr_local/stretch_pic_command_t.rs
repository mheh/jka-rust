#![allow(non_camel_case_types, non_snake_case)]

use super::shader_s::shader_s;

/// Raven `stretchPicCommand_t` — render-command to draw a stretched pic.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:2212-2219`
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

const _: () = assert!(core::mem::offset_of!(stretchPicCommand_t, commandId) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<stretchPicCommand_t>() == 48);
    assert!(core::mem::offset_of!(stretchPicCommand_t, shader) == 8);
    assert!(core::mem::offset_of!(stretchPicCommand_t, x) == 16);
    assert!(core::mem::offset_of!(stretchPicCommand_t, y) == 20);
    assert!(core::mem::offset_of!(stretchPicCommand_t, w) == 24);
    assert!(core::mem::offset_of!(stretchPicCommand_t, h) == 28);
    assert!(core::mem::offset_of!(stretchPicCommand_t, s1) == 32);
    assert!(core::mem::offset_of!(stretchPicCommand_t, t1) == 36);
    assert!(core::mem::offset_of!(stretchPicCommand_t, s2) == 40);
    assert!(core::mem::offset_of!(stretchPicCommand_t, t2) == 44);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<stretchPicCommand_t>() == 40);
    assert!(core::mem::offset_of!(stretchPicCommand_t, shader) == 4);
    assert!(core::mem::offset_of!(stretchPicCommand_t, x) == 8);
    assert!(core::mem::offset_of!(stretchPicCommand_t, y) == 12);
    assert!(core::mem::offset_of!(stretchPicCommand_t, w) == 16);
    assert!(core::mem::offset_of!(stretchPicCommand_t, h) == 20);
    assert!(core::mem::offset_of!(stretchPicCommand_t, s1) == 24);
    assert!(core::mem::offset_of!(stretchPicCommand_t, t1) == 28);
    assert!(core::mem::offset_of!(stretchPicCommand_t, s2) == 32);
    assert!(core::mem::offset_of!(stretchPicCommand_t, t2) == 36);
};
