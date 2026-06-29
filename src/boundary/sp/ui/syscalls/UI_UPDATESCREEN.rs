use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_UPDATESCREEN` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:180`
pub struct UiUpdatescreen;

impl OutboundSysCall for UiUpdatescreen {
    type Import = SpUiImport;
    /// Args source: `oracle/oracle/code/client/cl_ui.cpp:992-994`
    /// Output source: `oracle/oracle/code/client/cl_ui.cpp:992-994`
    /// (token-only RPC; wrapper calls `SCR_UpdateScreen()` and returns 0)
    type Args = ();
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_UPDATESCREEN;
}
