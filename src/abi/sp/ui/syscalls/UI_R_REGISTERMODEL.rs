use core::ffi::c_char;

use super::super::SpUiImport;
use crate::abi::generic::OutboundSysCall;
use crate::shared::qhandle_t;

/// `UI_R_REGISTERMODEL` SP UI imports syscall ABI token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:170`
pub struct UiRRegistermodel;

impl OutboundSysCall for UiRRegistermodel {
    type Import = SpUiImport;
    /// Args source: `oracle/oracle/code/ui/ui_public.h:46` and `oracle/oracle/code/client/cl_ui.cpp:391`.
    type Args = *const c_char;
    /// Output source: `oracle/oracle/code/ui/ui_public.h:46` and `oracle/oracle/code/client/cl_ui.cpp:391`.
    type Output = qhandle_t;

    const IMPORT: SpUiImport = SpUiImport::UI_R_REGISTERMODEL;
}
