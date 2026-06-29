use super::super::SpUiImport;
use crate::abi::generic::OutboundSysCall;

/// `UI_CM_LOADMODEL` SP UI imports syscall ABI token.
///
/// Raven: 30
/// Enum source: `oracle/oracle/code/ui/ui_public.h:182`
/// Args source: no wrapper or call site was found in `oracle/oracle/code/ui/ui_syscalls.cpp`,
/// `oracle/oracle/code/ui/ui_local.h`, `oracle/oracle/code/ui/ui_shared.cpp`, or
/// `oracle/oracle/code/client/cl_ui.cpp`.
/// Output source: no SP transport/return behavior found for this enum value.
/// Transport/switch source: no SP `UI_CM_LOADMODEL` case exists in `oracle/oracle/code/client/cl_ui.cpp`.
/// MP comparison source: `oracle/oracle/codemp/ui/ui_public.h:49` also declares
/// `UI_CM_LOADMODEL`, but no MP UI wrapper or client switch case was found.
///
/// TODO: Port args — SP `UI_CM_LOADMODEL` remains ambiguous because Raven only exposes the enum token.
/// TODO: Port output — no SP return behavior is visible in Raven source.
/// FIXME: create type for `UI_CM_LOADMODEL` args/output if a Raven wrapper or transport case is located.
pub struct UiCmLoadmodel;

impl OutboundSysCall for UiCmLoadmodel {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args - no Raven wrapper, call site, or switch case found.
    type Output = (); //TODO: Port output - no Raven wrapper, call site, or switch case found.

    const IMPORT: SpUiImport = SpUiImport::UI_CM_LOADMODEL;
}
