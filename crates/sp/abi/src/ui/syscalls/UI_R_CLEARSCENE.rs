use super::super::SpUiImport;
use abi_transport::generic::OutboundSysCall;

/// `UI_R_CLEARSCENE` SP UI imports syscall ABI token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:173`
pub struct UiRClearscene;

impl OutboundSysCall for UiRClearscene {
    type Import = SpUiImport;
    /// Args source: `oracle/oracle/code/ui/ui_public.h:173`,
    /// `oracle/oracle/codemp/ui/ui_syscalls.c:170-171`
    /// Transport/source switch evidence: `oracle/oracle/codemp/client/cl_ui.cpp:956-958`
    type Args = ();
    /// SP `cl_ui.cpp` does not define a direct `UI_R_CLEARSCENE` trap case in this branch;
    /// fallback transport evidence: `oracle/oracle/codemp/client/cl_ui.cpp:956-958`
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_R_CLEARSCENE;
}
