use core::ffi::c_char;

use super::super::SpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `UI_CVAR_RESET` SP UI imports syscall ABI token.
///
/// Enum source: `oracle/code/ui/ui_public.h:159`
/// Args source: `oracle/code/client/cl_ui.cpp` has no SP transport case;
/// `oracle/codemp/ui/ui_syscalls.c:59-60` provides the Raven wrapper signature.
/// Output source: no SP `cl_ui.cpp` transport output is present for this token;
/// `oracle/codemp/client/cl_ui.cpp:891-893`.
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:891-893` (fallback).
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

pub struct UiCvarReset;

impl OutboundSysCall for UiCvarReset {
    type Import = SpUiImport;
    type Args = UiCvarResetArgs;
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_CVAR_RESET;
}

impl EncodeSysCall for UiCvarReset {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.name())])
    }
}

impl DecodeSysCallReturn for UiCvarReset {
    fn decode_return(_word: isize) -> Self::Output {}
}
