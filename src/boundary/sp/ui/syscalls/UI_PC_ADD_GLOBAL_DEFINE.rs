use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_PC_ADD_GLOBAL_DEFINE` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:209`
pub struct UiPcAddGlobalDefine;

impl OutboundSysCall for UiPcAddGlobalDefine {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_PC_ADD_GLOBAL_DEFINE;
}
