use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_S_STOPBACKGROUNDTRACK` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:214`
pub struct UiSStopbackgroundtrack;

impl OutboundSysCall for UiSStopbackgroundtrack {
    type Import = SpUiImport;
    /// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:392-393`
    /// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:1173-1174`
    /// SP transport evidence indicates a token-only call in the UI engine (`S_StopBackgroundTrack(); return 0;`).
    type Args = ();
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_S_STOPBACKGROUNDTRACK;
}
