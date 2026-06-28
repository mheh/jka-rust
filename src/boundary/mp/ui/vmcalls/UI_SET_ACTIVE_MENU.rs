use super::super::MpUiExport;
use crate::boundary::generic::InboundVmCall;

/// `UI_SET_ACTIVE_MENU` MP UI exports vmMain boundary token.
///
/// Raven: qboolean UI_IsFullscreen( void );
/// Source: `oracle/oracle/codemp/ui/ui_public.h:237`
pub struct UiSetActiveMenu;

impl InboundVmCall for UiSetActiveMenu {
    type Command = MpUiExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpUiExport = MpUiExport::UI_SET_ACTIVE_MENU;
}
