use core::ffi::c_char;

use super::super::SpUiImport;
use abi_transport::generic::OutboundSysCall;
use sp_qshared::shared::qhandle_t;

/// `UI_R_REGISTERFONT` SP UI imports syscall ABI token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:207`
pub struct UiRRegisterfont;

impl OutboundSysCall for UiRRegisterfont {
    type Import = SpUiImport;
    /// Args source: `oracle/oracle/code/ui/ui_public.h:50` and `oracle/oracle/codemp/client/cl_ui.cpp:1132`.
    type Args = *const c_char;
    /// Output source: `oracle/oracle/code/ui/ui_public.h:50` and `oracle/oracle/codemp/client/cl_ui.cpp:1132`.
    type Output = qhandle_t;

    const IMPORT: SpUiImport = SpUiImport::UI_R_REGISTERFONT;
}
