use core::ffi::c_char;

use super::super::SpUiImport;
use abi_transport::generic::OutboundSysCall;
use sp_qshared::shared::qhandle_t;

/// `UI_R_REGISTERMODEL` SP UI imports syscall ABI token.
///
/// Source: `oracle/code/ui/ui_public.h:170`
pub struct UiRRegistermodel;

impl OutboundSysCall for UiRRegistermodel {
    type Import = SpUiImport;
    /// Args source: `oracle/code/ui/ui_public.h:46` and `oracle/code/client/cl_ui.cpp:391`.
    type Args = *const c_char;
    /// Output source: `oracle/code/ui/ui_public.h:46` and `oracle/code/client/cl_ui.cpp:391`.
    type Output = qhandle_t;

    const IMPORT: SpUiImport = SpUiImport::UI_R_REGISTERMODEL;
}
