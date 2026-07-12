#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_void;

use super::image_s::image_t;

/// Raven `subImageCommand_t` — render-command to upload a sub-image into a
/// loaded texture.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:2195-2201`
#[repr(C)]
pub struct subImageCommand_t {
    pub commandId: i32,
    pub image: *mut image_t,
    pub width: i32,
    pub height: i32,
    pub data: *mut c_void,
}

const _: () = assert!(core::mem::offset_of!(subImageCommand_t, commandId) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<subImageCommand_t>() == 32);
    assert!(core::mem::offset_of!(subImageCommand_t, image) == 8);
    assert!(core::mem::offset_of!(subImageCommand_t, width) == 16);
    assert!(core::mem::offset_of!(subImageCommand_t, height) == 20);
    assert!(core::mem::offset_of!(subImageCommand_t, data) == 24);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<subImageCommand_t>() == 20);
    assert!(core::mem::offset_of!(subImageCommand_t, image) == 4);
    assert!(core::mem::offset_of!(subImageCommand_t, width) == 8);
    assert!(core::mem::offset_of!(subImageCommand_t, height) == 12);
    assert!(core::mem::offset_of!(subImageCommand_t, data) == 16);
};
