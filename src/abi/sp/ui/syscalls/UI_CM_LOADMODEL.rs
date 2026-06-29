use super::super::SpUiImport;
use crate::abi::generic::OutboundSysCall;

/// `UI_CM_LOADMODEL` SP UI imports syscall ABI token.
///
/// Raven: 30
/// Enum source: `oracle/oracle/code/ui/ui_public.h:182`
/// Args source: not present in SP `oracle/oracle/code/ui/ui_syscalls.cpp` or
/// `oracle/oracle/code/client/cl_ui.cpp`; no callable payload evidence was located.
/// Output source: no SP transport/return behavior found for this enum value.
/// Transport/switch source: no SP `UI_CM_LOADMODEL` case exists in `oracle/oracle/code/client/cl_ui.cpp`.
///
/// TODO: SP `UI_CM_LOADMODEL` remains ambiguous because no parser/transport source was found.
/// FIXME: create type for `UI_CM_LOADMODEL` args/output when Raven type signatures are located.
pub struct UiCmLoadmodel;

impl OutboundSysCall for UiCmLoadmodel {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_CM_LOADMODEL;
}
