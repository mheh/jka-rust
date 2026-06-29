use core::ffi::c_int;

use super::super::MpUiImport;
use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_MEMORY_REMAINING`.
///
/// Raven wrapper: `return syscall( UI_MEMORY_REMAINING );`
/// Raven transport: `return Hunk_MemoryRemaining();`
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:342-343`
/// Args source: `oracle/oracle/codemp/ui/ui_local.h:982`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1119-1120`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct UiMemoryRemainingArgs;

impl UiMemoryRemainingArgs {
    pub const fn new() -> Self {
        Self
    }
}

/// `UI_MEMORY_REMAINING` MP UI imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:71`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:342-343`
/// Output source: `oracle/oracle/codemp/ui/ui_syscalls.c:342-343`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:1119-1120`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1119-1120`
pub struct UiMemoryRemaining;

impl OutboundSysCall for UiMemoryRemaining {
    type Import = MpUiImport;
    type Args = UiMemoryRemainingArgs;
    type Output = c_int;

    const IMPORT: MpUiImport = MpUiImport::UI_MEMORY_REMAINING;
}

impl EncodeSysCall for UiMemoryRemaining {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for UiMemoryRemaining {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
