use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_PC_ADD_GLOBAL_DEFINE` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:84`
pub struct UiPcAddGlobalDefine;

impl OutboundSysCall for UiPcAddGlobalDefine {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_PC_ADD_GLOBAL_DEFINE;
}
