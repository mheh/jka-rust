use super::super::MpUiExport;
use crate::boundary::generic::InboundVmCall;

/// `UI_MOUSE_EVENT` MP UI exports vmMain boundary token.
///
/// Raven: void	UI_KeyEvent( int key );
/// Source: `oracle/oracle/codemp/ui/ui_public.h:228`
pub struct UiMouseEvent;

impl InboundVmCall for UiMouseEvent {
    type Command = MpUiExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpUiExport = MpUiExport::UI_MOUSE_EVENT;
}
