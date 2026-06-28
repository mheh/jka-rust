use super::super::MpUiExport;
use crate::boundary::generic::InboundVmCall;

/// `UI_HASUNIQUECDKEY` MP UI exports vmMain boundary token.
///
/// Raven: void	UI_DrawConnectScreen( qboolean overlay );
/// Source: `oracle/oracle/codemp/ui/ui_public.h:245`
pub struct UiHasuniquecdkey;

impl InboundVmCall for UiHasuniquecdkey {
    type Command = MpUiExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpUiExport = MpUiExport::UI_HASUNIQUECDKEY;
}
