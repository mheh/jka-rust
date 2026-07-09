#![allow(non_camel_case_types, non_snake_case)]

/// Raven `scissorCommand_t` — render-command to set the GL scissor rect.
///
/// Type definition source: `oracle/code/renderer/tr_local.h:2033-2038`
#[repr(C)]
pub struct scissorCommand_t {
    pub commandId: i32,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

const _: () = assert!(core::mem::size_of::<scissorCommand_t>() == 20);
const _: () = assert!(core::mem::offset_of!(scissorCommand_t, commandId) == 0);
const _: () = assert!(core::mem::offset_of!(scissorCommand_t, x) == 4);
const _: () = assert!(core::mem::offset_of!(scissorCommand_t, y) == 8);
const _: () = assert!(core::mem::offset_of!(scissorCommand_t, w) == 12);
const _: () = assert!(core::mem::offset_of!(scissorCommand_t, h) == 16);
