#![allow(non_camel_case_types, non_snake_case)]

/// Raven `setModeCommand_t` — render-command to change the video mode.
///
/// Type definition source: `oracle/code/renderer/tr_local.h:2028-2031`
#[repr(C)]
pub struct setModeCommand_t {
    pub commandId: i32,
}

const _: () = assert!(core::mem::size_of::<setModeCommand_t>() == 4);
const _: () = assert!(core::mem::offset_of!(setModeCommand_t, commandId) == 0);
