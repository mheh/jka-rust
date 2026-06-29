use core::ffi::c_int;

use super::super::MpUiImport;
use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_MILLISECONDS`.
///
/// `trap_Milliseconds` takes no arguments; the transport carries no payload
/// words after the import token.
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:29`
/// Transport source: `oracle/oracle/codemp/ui/ui_syscalls.c:30`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:694`
#[derive(Debug, Default)]
pub struct CgMillisecondsArgs;

impl CgMillisecondsArgs {
    pub const fn new() -> Self {
        Self
    }
}

/// `UI_MILLISECONDS` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:59`
/// Output source: `oracle/oracle/codemp/ui/ui_syscalls.c:29`
/// Output source: `oracle/oracle/codemp/ui/ui_syscalls.c:30`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:695`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:694`
pub struct CgMilliseconds;

impl OutboundSysCall for CgMilliseconds {
    type Import = MpUiImport;
    type Args = CgMillisecondsArgs;
    type Output = c_int;

    const IMPORT: MpUiImport = MpUiImport::UI_MILLISECONDS;
}

impl EncodeSysCall for CgMilliseconds {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for CgMilliseconds {
    // `trap_Milliseconds` returns `int`; the engine's return word is that value.
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
