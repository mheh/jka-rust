use super::super::MpUiExport;
use crate::boundary::generic::InboundVmCall;

/// `UI_SHUTDOWN` MP UI exports vmMain boundary token.
///
/// Raven: void	UI_Init( void );
/// Source: `oracle/oracle/codemp/ui/ui_public.h:222`
pub struct UiShutdown;

impl InboundVmCall for UiShutdown {
    type Command = MpUiExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpUiExport = MpUiExport::UI_SHUTDOWN;
}
