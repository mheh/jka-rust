use core::ffi::c_char;

use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_PRINT` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:153`
pub struct UiPrint;

impl OutboundSysCall for UiPrint {
    type Import = SpUiImport;
    /// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:22`
    /// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:862`
    type Args = *const c_char;
    /// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:863`
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_PRINT;
}
