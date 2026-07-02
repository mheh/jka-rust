#![allow(non_camel_case_types, non_snake_case)]

/// Raven `swapBuffersCommand_t` — render-command to swap the front/back
/// buffers.
///
/// Type definition source: `oracle/oracle/code/renderer/tr_local.h:2000-2002`
#[repr(C)]
pub struct swapBuffersCommand_t {
    pub commandId: i32,
}

const _: () = assert!(core::mem::size_of::<swapBuffersCommand_t>() == 4);
const _: () = assert!(core::mem::offset_of!(swapBuffersCommand_t, commandId) == 0);
