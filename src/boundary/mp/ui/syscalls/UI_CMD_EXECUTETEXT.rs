use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_CMD_EXECUTETEXT` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:30`
pub struct UiCmdExecutetext;

impl OutboundSysCall for UiCmdExecutetext {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_CMD_EXECUTETEXT;
}
