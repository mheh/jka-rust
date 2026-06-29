use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_CM_LOADMODEL` SP UI imports syscall boundary token.
///
/// Raven: 30
/// Enum value source: `oracle/oracle/code/ui/ui_public.h:182`
/// Args source: not present in SP `oracle/oracle/code/ui/ui_syscalls.cpp` or
/// `oracle/oracle/code/client/cl_ui.cpp`; no callable payload evidence was located.
/// Output source: not present in SP `oracle/oracle/code/client/cl_ui.cpp`.
/// Transport/switch source: not present in SP `oracle/oracle/code/client/cl_ui.cpp`.
/// TODO: SP `UI_CM_LOADMODEL` remains ambiguous due absent engine switch evidence.
pub struct UiCmLoadmodel;

impl OutboundSysCall for UiCmLoadmodel {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_CM_LOADMODEL;
}
