use core::ffi::c_int;

use super::super::MpUiImport;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `UI_LAN_GETPINGQUEUECOUNT` MP UI imports syscall ABI token.
///
/// Raven wrapper: `return syscall( UI_LAN_GETPINGQUEUECOUNT );`
/// Raven transport: `return LAN_GetPingQueueCount();`
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:65`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:286-287`
/// Output source: `oracle/oracle/codemp/ui/ui_local.h:968`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1070-1071`
pub struct UiLanGetpingqueuecount;

impl OutboundSysCall for UiLanGetpingqueuecount {
    type Import = MpUiImport;
    type Args = ();
    type Output = c_int;

    const IMPORT: MpUiImport = MpUiImport::UI_LAN_GETPINGQUEUECOUNT;
}

impl EncodeSysCall for UiLanGetpingqueuecount {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for UiLanGetpingqueuecount {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
