use core::ffi::c_char;

use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;
use crate::ffi::types::qboolean;

/// `UI_S_STARTBACKGROUNDTRACK` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:215`
pub struct UiSStartbackgroundtrack;

impl OutboundSysCall for UiSStartbackgroundtrack {
    type Import = SpUiImport;
    /// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:396-397`
    /// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:1176-1177`
    /// SP transport case drops the third argument and passes `qfalse`.
    /// The ABI keeps the third value as `qboolean` for signature parity.
    type Args = (*const c_char, *const c_char, qboolean);
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_S_STARTBACKGROUNDTRACK;
}
