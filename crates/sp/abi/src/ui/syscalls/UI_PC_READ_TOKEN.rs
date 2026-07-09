use core::ffi::c_int;

use super::super::SpUiImport;
use abi_transport::generic::OutboundSysCall;
use sp_qshared::shared::pc_token_t;

/// `UI_PC_READ_TOKEN` SP UI imports syscall ABI token.
///
/// Raven: 60
/// Source: `oracle/code/ui/ui_public.h:212`
pub struct UiPcReadToken;

impl OutboundSysCall for UiPcReadToken {
    type Import = SpUiImport;
    /// SP `client/cl_ui.cpp` has no active `UI_PC_READ_TOKEN` dispatch case.
    /// Fallback transport shape from MP parity:
    /// `oracle/codemp/ui/ui_syscalls.c:374-375` and
    /// `oracle/codemp/client/cl_ui.cpp:1163-1164`.
    ///
    /// Args source: `(int handle, pc_token_t *pc_token)`.
    type Args = (c_int, *mut pc_token_t);
    /// Returns `int` from botlib token parser call; fallback source:
    /// `oracle/codemp/ui/ui_syscalls.c:374-375`.
    type Output = c_int;

    const IMPORT: SpUiImport = SpUiImport::UI_PC_READ_TOKEN;
}
