use core::ffi::c_int;

use super::super::SpUiImport;
use abi_transport::generic::OutboundSysCall;

/// `UI_PC_FREE_SOURCE` SP UI imports syscall ABI token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:211`
pub struct UiPcFreeSource;

impl OutboundSysCall for UiPcFreeSource {
    type Import = SpUiImport;
    /// SP `client/cl_ui.cpp` has no active `UI_PC_FREE_SOURCE` dispatch case.
    /// Fallback transport shape from MP parity:
    /// `oracle/oracle/codemp/ui/ui_syscalls.c:370-371` and
    /// `oracle/oracle/codemp/client/cl_ui.cpp:1162`.
    ///
    /// Args source: single integer handle (`args[1]`).
    type Args = c_int;
    /// Returns `int` from botlib handle free call; fallback source:
    /// `oracle/oracle/codemp/ui/ui_syscalls.c:370-371`.
    type Output = c_int;

    const IMPORT: SpUiImport = SpUiImport::UI_PC_FREE_SOURCE;
}
