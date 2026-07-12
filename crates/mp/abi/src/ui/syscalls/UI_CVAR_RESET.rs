use core::ffi::c_char;

use super::super::MpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_CVAR_RESET`.
///
/// Raven wrapper: `syscall( UI_CVAR_RESET, name );`
/// Raven transport: `Cvar_Reset( (const char *)VMA(1) );`
///
/// Args source: `oracle/codemp/ui/ui_syscalls.c:59-60`
/// Args source: `oracle/codemp/ui/ui_local.h:924`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:891-893`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiCvarResetArgs {
    name: *const c_char,
}

impl UiCvarResetArgs {
    pub const fn new(name: *const c_char) -> Self {
        Self { name }
    }

    pub const fn name(&self) -> *const c_char {
        self.name
    }
}

/// `UI_CVAR_RESET` MP UI imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/ui/ui_public.h:25`
/// Args source: `oracle/codemp/ui/ui_syscalls.c:59-60`
/// Output source: `oracle/codemp/client/cl_ui.cpp:891-893`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:891-893`
pub struct UiCvarReset;

impl OutboundSysCall for UiCvarReset {
    type Import = MpUiImport;
    type Args = UiCvarResetArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_CVAR_RESET;
}

impl EncodeSysCall for UiCvarReset {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.name())])
    }
}

impl DecodeSysCallReturn for UiCvarReset {
    fn decode_return(_word: isize) -> Self::Output {}
}
