use super::super::MpUiExport;
use crate::boundary::generic::InboundVmCall;

/// `UI_REFRESH` MP UI exports vmMain boundary token.
///
/// Raven: void	UI_MouseEvent( int dx, int dy );
/// Source: `oracle/oracle/codemp/ui/ui_public.h:231`
pub struct UiRefresh;

impl InboundVmCall for UiRefresh {
    type Command = MpUiExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpUiExport = MpUiExport::UI_REFRESH;
}
