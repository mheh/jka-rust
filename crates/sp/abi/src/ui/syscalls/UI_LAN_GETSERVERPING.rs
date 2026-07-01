use core::ffi::c_int;

use super::super::SpUiImport;
use abi_transport::generic::OutboundSysCall;

/// `UI_LAN_GETSERVERPING` SP UI imports syscall ABI token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:235`
pub struct UiLanGetserverping;

impl OutboundSysCall for UiLanGetserverping {
    type Import = SpUiImport;
    /// Raven wrapper: `return syscall( UI_LAN_GETSERVERPING, source, n );`
    /// Transport source: `oracle/oracle/codemp/client/cl_ui.cpp:1096-1097`
    /// SP transport path in `oracle/oracle/code/client/cl_ui.cpp` has no `UI_LAN_GETSERVERPING` case.
    ///
    /// Args source: `oracle/oracle/codemp/client/cl_ui.cpp:1096-1097`
    /// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:1096-1097`
    type Args = (c_int, c_int);
    /// Returns ping time as integer.
    /// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:1096-1097`
    type Output = c_int;

    const IMPORT: SpUiImport = SpUiImport::UI_LAN_GETSERVERPING;
}
