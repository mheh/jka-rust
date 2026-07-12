use core::ffi::{c_char, c_int};

use super::super::SpUiImport;
use abi_transport::generic::OutboundSysCall;

/// `UI_LAN_GETSERVERADDRESSSTRING` SP UI imports syscall ABI token.
///
/// Source: `oracle/code/ui/ui_public.h:218`
pub struct UiLanGetserveraddressstring;

impl OutboundSysCall for UiLanGetserveraddressstring {
    type Import = SpUiImport;
    /// Raven wrapper: `syscall( UI_LAN_GETSERVERADDRESSSTRING, source, n, buf, buflen );`
    /// Transport source: `oracle/codemp/client/cl_ui.cpp:1088-1090`
    /// SP transport path in `oracle/code/client/cl_ui.cpp` has no `UI_LAN_GETSERVERADDRESSSTRING` case.
    ///
    /// Args source: `oracle/codemp/client/cl_ui.cpp:1088-1090`
    /// Output source: `oracle/codemp/client/cl_ui.cpp:1090`
    type Args = (c_int, c_int, *mut c_char, c_int);
    /// Void return (implemented as `return 0;` in the transport path)
    /// Output source: `oracle/codemp/client/cl_ui.cpp:1090`
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_LAN_GETSERVERADDRESSSTRING;
}
