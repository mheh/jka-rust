use core::ffi::{c_char, c_int};

use super::super::SpUiImport;
use abi_transport::generic::OutboundSysCall;

/// `UI_LAN_GETPINGINFO` SP UI imports syscall ABI token.
///
/// Source: `oracle/code/ui/ui_public.h:201`
pub struct UiLanGetpinginfo;

impl OutboundSysCall for UiLanGetpinginfo {
    type Import = SpUiImport;
    /// Raven wrapper: `syscall( UI_LAN_GETPINGINFO, n, buf, buflen );`
    /// Transport source: `oracle/codemp/client/cl_ui.cpp:1081-1083`
    /// SP transport path in `oracle/code/client/cl_ui.cpp` has no `UI_LAN_GETPINGINFO` case.
    ///
    /// Args source: `oracle/codemp/client/cl_ui.cpp:1081-1083`
    /// Output source: `oracle/codemp/client/cl_ui.cpp:1083`
    type Args = (c_int, *mut c_char, c_int);
    /// Void return (implemented as `return 0;` in the transport path)
    /// Output source: `oracle/codemp/client/cl_ui.cpp:1083`
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_LAN_GETPINGINFO;
}
