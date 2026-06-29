use core::ffi::{c_char, c_int};

use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_PC_LOAD_SOURCE` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:210`
pub struct UiPcLoadSource;

impl OutboundSysCall for UiPcLoadSource {
    type Import = SpUiImport;
    /// SP `client/cl_ui.cpp` has no active `UI_PC_LOAD_SOURCE` dispatch case.
    /// Fallback transport shape from MP parity:
    /// `oracle/oracle/codemp/ui/ui_syscalls.c:366-367` and
    /// `oracle/oracle/codemp/client/cl_ui.cpp:1160`.
    ///
    /// Args source: const filename pointer (`args[1]`).
    type Args = *const c_char;
    /// Returns `int` from `PC_LoadSourceHandle`; fallback source:
    /// `oracle/oracle/codemp/ui/ui_syscalls.c:366-367`.
    type Output = c_int;

    const IMPORT: SpUiImport = SpUiImport::UI_PC_LOAD_SOURCE;
}
