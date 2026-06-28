use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_PC_REMOVE_ALL_GLOBAL_DEFINES` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:90`
pub struct UiPcRemoveAllGlobalDefines;

impl OutboundSysCall for UiPcRemoveAllGlobalDefines {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_PC_REMOVE_ALL_GLOBAL_DEFINES;
}
