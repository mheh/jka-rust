use super::super::MpUiExport;
use crate::boundary::generic::InboundVmCall;

/// `UI_CONSOLE_COMMAND` MP UI exports vmMain boundary token.
///
/// Raven: void	UI_SetActiveMenu( uiMenuCommand_t menu );
/// Source: `oracle/oracle/codemp/ui/ui_public.h:240`
pub struct UiConsoleCommand;

impl InboundVmCall for UiConsoleCommand {
    type Command = MpUiExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpUiExport = MpUiExport::UI_CONSOLE_COMMAND;
}
