#![allow(non_camel_case_types, non_snake_case)]

/// Raven `TCGIncomingConsoleCommand` — incoming console command buffer.
///
/// Type definition source: `oracle/codemp/cgame/cg_public.h:507-510`
#[repr(C)]
pub struct TCGIncomingConsoleCommand {
    pub conCommand: [u8; 1024],
}

const _: () = assert!(core::mem::size_of::<TCGIncomingConsoleCommand>() == 1024);
const _: () = assert!(core::mem::offset_of!(TCGIncomingConsoleCommand, conCommand) == 0);
