#![allow(non_camel_case_types, non_snake_case)]

/// Raven `endFrameCommand_t` — end-of-frame render command.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:2207-2210`
#[repr(C)]
pub struct endFrameCommand_t {
    pub commandId: i32,
    pub buffer: i32,
}

const _: () = assert!(core::mem::size_of::<endFrameCommand_t>() == 8);
const _: () = assert!(core::mem::offset_of!(endFrameCommand_t, commandId) == 0);
const _: () = assert!(core::mem::offset_of!(endFrameCommand_t, buffer) == 4);
