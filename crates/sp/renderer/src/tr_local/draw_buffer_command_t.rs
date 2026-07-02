#![allow(non_camel_case_types, non_snake_case)]

/// Raven `drawBufferCommand_t`.
///
/// Type definition source: `oracle/oracle/code/renderer/tr_local.h:1987-1990`
#[repr(C)]
pub struct drawBufferCommand_t {
    pub commandId: i32,
    pub buffer: i32,
}

const _: () = assert!(core::mem::size_of::<drawBufferCommand_t>() == 8);
const _: () = assert!(core::mem::offset_of!(drawBufferCommand_t, commandId) == 0);
const _: () = assert!(core::mem::offset_of!(drawBufferCommand_t, buffer) == 4);
