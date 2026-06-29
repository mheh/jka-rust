use core::ffi::{c_char, c_int};

use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_LAN_GETPING` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:200`
pub struct UiLanGetping;

impl OutboundSysCall for UiLanGetping {
    type Import = SpUiImport;
    /// Raven wrapper: `syscall( UI_LAN_GETPING, n, buf, buflen, pingtime );`
    /// Transport source: `oracle/oracle/codemp/client/cl_ui.cpp:1077-1079`
    /// SP transport path in `oracle/oracle/code/client/cl_ui.cpp` has no `UI_LAN_GETPING` case.
    ///
    /// Args source: `oracle/oracle/codemp/client/cl_ui.cpp:1077-1079`
    /// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:1079`
    type Args = (c_int, *mut c_char, c_int, *mut c_int);
    /// Void return (implemented as `return 0;` in the transport path)
    /// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:1079`
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_LAN_GETPING;
}
