use super::super::SpUiImport;
use abi_transport::generic::OutboundSysCall;

/// `UI_CM_LOADMODEL` SP UI imports syscall ABI token.
///
/// Enum value source: `oracle/code/ui/ui_public.h:182`
/// Args source: no wrapper or call site found in Raven source.
/// Output source: no SP transport/return behavior found in Raven source.
/// Transport/switch source: no SP `UI_CM_LOADMODEL` case exists in `oracle/code/client/cl_ui.cpp`.
///
/// Raven wrapper: not found.
/// Raven transport: not found.
///
/// Raven enum source: `oracle/code/ui/ui_public.h:182`
/// Raven wrapper search sources: `oracle/code/ui/ui_syscalls.cpp`,
/// `oracle/code/ui/ui_local.h`, `oracle/code/ui/ui_shared.cpp`
/// Raven transport search source: `oracle/code/client/cl_ui.cpp`
/// Raven MP comparison source: `oracle/codemp/ui/ui_public.h:49`
///
/// NOTE: does not appear to be used anywhere.
pub struct UiCmLoadmodel;

impl OutboundSysCall for UiCmLoadmodel {
    type Import = SpUiImport;
    type Args = ();
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_CM_LOADMODEL;
}
