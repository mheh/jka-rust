use core::ffi::{c_char, c_int};

use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_PC_ADD_GLOBAL_DEFINE` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:209`
pub struct UiPcAddGlobalDefine;

impl OutboundSysCall for UiPcAddGlobalDefine {
    type Import = SpUiImport;
    /// SP `client/cl_ui.cpp` has no active `UI_PC_ADD_GLOBAL_DEFINE` dispatch case.
    /// Transport shape is inferred from MP trap parity:
    /// `oracle/oracle/codemp/ui/ui_syscalls.c:362-363` and
    /// `oracle/oracle/codemp/client/cl_ui.cpp:1158`.
    ///
    /// Args source: `(char *)VMA(1)` only.
    type Args = *mut c_char;
    /// MP parity returns `int` from `PC_AddGlobalDefine`, and fallback evidence is in
    /// `oracle/oracle/codemp/ui/ui_syscalls.c:362-363`.
    type Output = c_int;

    const IMPORT: SpUiImport = SpUiImport::UI_PC_ADD_GLOBAL_DEFINE;
}
