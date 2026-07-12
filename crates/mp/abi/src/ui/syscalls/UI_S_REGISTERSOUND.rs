use core::ffi::{c_char, c_int};

use super::super::MpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_S_REGISTERSOUND`.
///
/// Raven wrapper: `return syscall( UI_S_REGISTERSOUND, sample );`
/// Raven transport: `return S_RegisterSound( (const char *)VMA(1) );`
///
/// Args source: `oracle/codemp/ui/ui_syscalls.c:214-215`
/// Args source: `oracle/codemp/ui/ui_local.h:950`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:1000-1001`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiSRegistersoundArgs {
    sample: *const c_char,
}

impl UiSRegistersoundArgs {
    pub const fn new(sample: *const c_char) -> Self {
        Self { sample }
    }
}

/// `UI_S_REGISTERSOUND` MP UI imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/ui/ui_public.h:50`
/// Args source: `oracle/codemp/ui/ui_syscalls.c:214-215`
/// Output source: `oracle/codemp/ui/ui_local.h:950`
/// Output source: `oracle/codemp/client/cl_ui.cpp:1000-1001`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:1000-1001`
pub struct UiSRegistersound;

impl OutboundSysCall for UiSRegistersound {
    type Import = MpUiImport;
    type Args = UiSRegistersoundArgs;
    type Output = c_int;

    const IMPORT: MpUiImport = MpUiImport::UI_S_REGISTERSOUND;
}

impl EncodeSysCall for UiSRegistersound {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.sample)])
    }
}

impl DecodeSysCallReturn for UiSRegistersound {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
