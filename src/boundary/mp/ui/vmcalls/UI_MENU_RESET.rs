use super::super::MpUiExport;
use crate::boundary::generic::InboundVmCall;

/// `UI_MENU_RESET` MP UI exports vmMain boundary token.
///
/// Raven: if !overlay, the background will be drawn, otherwise it will be
/// Raven: overlayed over whatever the cgame has drawn.
/// Raven: a GetClientState syscall will be made to get the current strings
/// Source: `oracle/oracle/codemp/ui/ui_public.h:250`
pub struct UiMenuReset;

impl InboundVmCall for UiMenuReset {
    type Command = MpUiExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpUiExport = MpUiExport::UI_MENU_RESET;
}
