use super::super::MpUiExport;
use crate::boundary::generic::InboundVmCall;

/// `UI_DRAW_CONNECT_SCREEN` MP UI exports vmMain boundary token.
///
/// Raven: qboolean UI_ConsoleCommand( int realTime );
/// Source: `oracle/oracle/codemp/ui/ui_public.h:243`
pub struct UiDrawConnectScreen;

impl InboundVmCall for UiDrawConnectScreen {
    type Command = MpUiExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpUiExport = MpUiExport::UI_DRAW_CONNECT_SCREEN;
}
