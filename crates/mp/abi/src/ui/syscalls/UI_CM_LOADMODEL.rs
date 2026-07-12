use super::super::MpUiImport;
use abi_transport::generic::OutboundSysCall;

/// `UI_CM_LOADMODEL` MP UI imports syscall ABI token.
///
/// Raven: no callable payload was located in the tracked MP UI sources.
/// Enum value source: `oracle/codemp/ui/ui_public.h:49`
/// Args source: no parser/trap wrapper location for `UI_CM_LOADMODEL` in
/// `oracle/codemp/ui/ui_syscalls.c`, `oracle/codemp/ui/ui_local.h`, or `oracle/codemp/ui/ui_shared.h`.
/// Output source: no handler/return path located in the same MP UI syscall sources; enum exists without transport.
/// Transport/switch source: no `UI_CM_LOADMODEL` case appears in `oracle/codemp/client/cl_ui.cpp`.
/// Transport/call-site source: no client call-site was found for `UI_CM_LOADMODEL` in tracked MP sources; transport may be unused/unimplemented in this tree.
pub struct UiCmLoadmodel;

impl OutboundSysCall for UiCmLoadmodel {
    type Import = MpUiImport;
    type Args = ();
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_CM_LOADMODEL;
}
