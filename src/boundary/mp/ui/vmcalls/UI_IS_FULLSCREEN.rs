use super::super::MpUiExport;
use crate::boundary::generic::InboundVmCall;

/// `UI_IS_FULLSCREEN` MP UI exports vmMain boundary token.
///
/// Raven: void	UI_Refresh( int time );
/// Source: `oracle/oracle/codemp/ui/ui_public.h:234`
pub struct UiIsFullscreen;

impl InboundVmCall for UiIsFullscreen {
    type Command = MpUiExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpUiExport = MpUiExport::UI_IS_FULLSCREEN;
}
