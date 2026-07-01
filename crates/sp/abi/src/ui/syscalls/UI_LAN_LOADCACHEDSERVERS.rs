use super::super::SpUiImport;
use abi_transport::generic::OutboundSysCall;

/// `UI_LAN_LOADCACHEDSERVERS` SP UI imports syscall ABI token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:223`
pub struct UiLanLoadcachedservers;

impl OutboundSysCall for UiLanLoadcachedservers {
    type Import = SpUiImport;
    /// Args source: `oracle/oracle/codemp/client/cl_ui.cpp:1055` (SP transport path in
    /// `oracle/oracle/code/client/cl_ui.cpp` does not define this case)
    /// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:1055-1057`
    type Args = ();
    /// No return value (void path).
    /// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:1057`
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_LAN_LOADCACHEDSERVERS;
}
