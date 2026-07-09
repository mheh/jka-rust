use core::ffi::c_char;

use super::super::SpUiImport;
use abi_transport::generic::OutboundSysCall;

/// `UI_PRINT` SP UI imports syscall ABI token.
///
/// Source: `oracle/code/ui/ui_public.h:153`
pub struct UiPrint;

impl OutboundSysCall for UiPrint {
    type Import = SpUiImport;
    /// Args source: `oracle/codemp/ui/ui_syscalls.c:22`
    /// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:862`
    type Args = *const c_char;
    /// Output source: `oracle/codemp/client/cl_ui.cpp:863`
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_PRINT;
}
