use core::ffi::c_char;

use super::super::SpUiImport;
use crate::abi::generic::OutboundSysCall;
use crate::codemp::game::q_shared_h::qhandle_t;

/// `UI_R_REGISTERSHADERNOMIP` SP UI imports syscall ABI token.
///
/// Raven: 20
/// Source: `oracle/oracle/code/ui/ui_public.h:172`
pub struct UiRRegistershadernomip;

impl OutboundSysCall for UiRRegistershadernomip {
    type Import = SpUiImport;
    /// Args source: `oracle/oracle/code/ui/ui_public.h:49` and `oracle/oracle/code/client/cl_ui.cpp:394`.
    type Args = *const c_char;
    /// Output source: `oracle/oracle/code/ui/ui_public.h:49` and `oracle/oracle/code/client/cl_ui.cpp:394`.
    type Output = qhandle_t;

    const IMPORT: SpUiImport = SpUiImport::UI_R_REGISTERSHADERNOMIP;
}
