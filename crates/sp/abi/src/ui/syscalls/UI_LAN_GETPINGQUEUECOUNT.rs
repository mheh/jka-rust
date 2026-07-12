use core::ffi::c_int;

use super::super::SpUiImport;
use abi_transport::generic::OutboundSysCall;

/// `UI_LAN_GETPINGQUEUECOUNT` SP UI imports syscall ABI token.
///
/// Source: `oracle/code/ui/ui_public.h:198`
pub struct UiLanGetpingqueuecount;

impl OutboundSysCall for UiLanGetpingqueuecount {
    type Import = SpUiImport;
    /// Args source: `oracle/codemp/client/cl_ui.cpp:1070` (SP transport path in
    /// `oracle/code/client/cl_ui.cpp` does not define this case)
    /// Output source: `oracle/codemp/client/cl_ui.cpp:1071`
    type Args = ();
    /// Output is an integer count.
    /// Output source: `oracle/codemp/client/cl_ui.cpp:1071`
    type Output = c_int;

    const IMPORT: SpUiImport = SpUiImport::UI_LAN_GETPINGQUEUECOUNT;
}
