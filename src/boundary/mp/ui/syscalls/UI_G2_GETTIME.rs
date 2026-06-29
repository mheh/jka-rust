use core::ffi::c_int;

use super::super::MpUiImport;
use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_G2_GETTIME`.
///
/// Raven wrapper: `int trap_G2API_GetTime(void)`.
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:624-626`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1378-1379`
#[derive(Debug, Default)]
pub struct UiG2GettimeArgs;

impl UiG2GettimeArgs {
    pub const fn new() -> Self {
        Self
    }
}

/// `UI_G2_GETTIME` MP UI imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:169`
/// Enum comment source: `oracle/oracle/codemp/ui/ui_public.h:169`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:624-626`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:1378-1379`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1378-1379`
pub struct UiG2Gettime;

impl OutboundSysCall for UiG2Gettime {
    type Import = MpUiImport;
    type Args = UiG2GettimeArgs;
    type Output = c_int;

    const IMPORT: MpUiImport = MpUiImport::UI_G2_GETTIME;
}

impl EncodeSysCall for UiG2Gettime {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for UiG2Gettime {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
