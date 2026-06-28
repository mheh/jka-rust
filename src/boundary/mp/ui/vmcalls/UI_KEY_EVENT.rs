use super::super::MpUiExport;
use crate::boundary::generic::InboundVmCall;

/// `UI_KEY_EVENT` MP UI exports vmMain boundary token.
///
/// Raven: void	UI_Shutdown( void );
/// Source: `oracle/oracle/codemp/ui/ui_public.h:225`
pub struct UiKeyEvent;

impl InboundVmCall for UiKeyEvent {
    type Command = MpUiExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpUiExport = MpUiExport::UI_KEY_EVENT;
}
