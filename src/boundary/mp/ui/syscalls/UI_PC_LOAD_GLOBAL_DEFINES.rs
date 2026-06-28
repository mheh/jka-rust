use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_PC_LOAD_GLOBAL_DEFINES` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:89`
pub struct UiPcLoadGlobalDefines;

impl OutboundSysCall for UiPcLoadGlobalDefines {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_PC_LOAD_GLOBAL_DEFINES;
}
