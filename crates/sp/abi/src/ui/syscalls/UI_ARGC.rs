use core::ffi::c_int;

use super::super::SpUiImport;
use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `UI_ARGC` SP UI imports syscall ABI token.
///
/// Enum value source: `oracle/code/ui/ui_public.h:162`
/// Args source: `oracle/code/qcommon/qcommon.h:289`
/// Output source: `oracle/code/qcommon/qcommon.h:289`
/// Transport/switch source: `oracle/code/client/cl_ui.cpp:218`
pub struct UiArgc;

#[derive(Debug, Default)]
pub struct UiArgcArgs;

impl UiArgcArgs {
    pub const fn new() -> Self {
        Self
    }
}

impl OutboundSysCall for UiArgc {
    type Import = SpUiImport;
    type Args = UiArgcArgs;
    type Output = c_int;

    const IMPORT: SpUiImport = SpUiImport::UI_ARGC;
}

impl EncodeSysCall for UiArgc {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for UiArgc {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
