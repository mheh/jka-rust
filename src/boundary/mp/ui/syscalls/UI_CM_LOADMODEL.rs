use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_CM_LOADMODEL` MP UI imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:49`
/// Args source: no MP UI trap wrapper or engine switch arm found in
/// `oracle/oracle/codemp/ui/ui_syscalls.c`, `oracle/oracle/codemp/ui/ui_local.h`,
/// or `oracle/oracle/codemp/client/cl_ui.cpp`.
/// Output source: no MP UI trap wrapper or engine switch arm found in
/// `oracle/oracle/codemp/ui/ui_syscalls.c`, `oracle/oracle/codemp/ui/ui_local.h`,
/// or `oracle/oracle/codemp/client/cl_ui.cpp`.
pub struct UiCmLoadmodel;

impl OutboundSysCall for UiCmLoadmodel {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_CM_LOADMODEL;
}
