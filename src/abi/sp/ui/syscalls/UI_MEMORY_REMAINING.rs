use core::ffi::c_int;

use super::super::SpUiImport;
use crate::abi::generic::OutboundSysCall;

/// `UI_MEMORY_REMAINING` SP UI imports syscall ABI token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:204`
///
/// Args source: no SP-side argument payload is present in `oracle/oracle/code/client/cl_ui.cpp`
/// (`UI_MEMORY_REMAINING` has no `case` there); fallback from MP ABI:
/// `oracle/oracle/codemp/ui/ui_syscalls.c:342` and `oracle/oracle/codemp/client/cl_ui.cpp:1119-1120`.
///
/// Output source: fallback from MP ABI:
/// `oracle/oracle/codemp/ui/ui_syscalls.c:342-343` and `oracle/oracle/codemp/client/cl_ui.cpp:1119-1120`.
pub struct UiMemoryRemaining;

impl OutboundSysCall for UiMemoryRemaining {
    type Import = SpUiImport;
    /// No SP `UI_MEMORY_REMAINING` arguments: token-only RPC.
    type Args = ();
    /// Returns `int` from `Hunk_MemoryRemaining`.
    type Output = c_int;

    const IMPORT: SpUiImport = SpUiImport::UI_MEMORY_REMAINING;
}
