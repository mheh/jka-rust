#![allow(non_camel_case_types, non_snake_case)]

use super::image_s::image_t;

/// Raven `subImageCommand_t` — render-command to upload a sub-rectangle of
/// pixel `data` into `image` at `width` x `height`.
///
/// Type definition source: `oracle/oracle/code/renderer/tr_local.h:1992-1998`
#[repr(C)]
pub struct subImageCommand_t {
    pub commandId: i32,
    pub image: *mut image_t,
    pub width: i32,
    pub height: i32,
    pub data: *mut core::ffi::c_void,
}

const _: () = assert!(core::mem::size_of::<subImageCommand_t>() == 32);
const _: () = assert!(core::mem::offset_of!(subImageCommand_t, commandId) == 0);
const _: () = assert!(core::mem::offset_of!(subImageCommand_t, image) == 8);
const _: () = assert!(core::mem::offset_of!(subImageCommand_t, width) == 16);
const _: () = assert!(core::mem::offset_of!(subImageCommand_t, height) == 20);
const _: () = assert!(core::mem::offset_of!(subImageCommand_t, data) == 24);
