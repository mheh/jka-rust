#![allow(non_camel_case_types, non_snake_case)]

/// Raven `setColorCommand_t` — render-command to change the current draw color.
///
/// Type definition source: `oracle/oracle/codemp/renderer/tr_local.h:2185-2188`
#[repr(C)]
pub struct setColorCommand_t {
    pub commandId: i32,
    pub color: [f32; 4],
}

const _: () = assert!(core::mem::size_of::<setColorCommand_t>() == 20);
const _: () = assert!(core::mem::offset_of!(setColorCommand_t, commandId) == 0);
const _: () = assert!(core::mem::offset_of!(setColorCommand_t, color) == 4);
