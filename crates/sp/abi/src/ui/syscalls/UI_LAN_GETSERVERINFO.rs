use core::ffi::{c_char, c_int};

use super::super::SpUiImport;
use abi_transport::generic::OutboundSysCall;

/// `UI_LAN_GETSERVERINFO` SP UI imports syscall ABI token.
///
/// Source: `oracle/code/ui/ui_public.h:219`
pub struct UiLanGetserverinfo;

impl OutboundSysCall for UiLanGetserverinfo {
    type Import = SpUiImport;
    /// Raven wrapper: `syscall( UI_LAN_GETSERVERINFO, source, n, buf, buflen );`
    /// Transport source: `oracle/codemp/client/cl_ui.cpp:1092-1094`
    /// SP transport path in `oracle/code/client/cl_ui.cpp` has no `UI_LAN_GETSERVERINFO` case.
    ///
    /// Args source: `oracle/codemp/client/cl_ui.cpp:1092-1094`
    /// Output source: `oracle/codemp/client/cl_ui.cpp:1094`
    type Args = (c_int, c_int, *mut c_char, c_int);
    /// Void return (implemented as `return 0;` in the transport path)
    /// Output source: `oracle/codemp/client/cl_ui.cpp:1094`
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_LAN_GETSERVERINFO;
}
