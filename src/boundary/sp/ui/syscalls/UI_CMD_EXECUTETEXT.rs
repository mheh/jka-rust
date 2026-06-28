use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_CMD_EXECUTETEXT` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:164`
pub struct UiCmdExecutetext;

impl OutboundSysCall for UiCmdExecutetext {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_CMD_EXECUTETEXT;
}
