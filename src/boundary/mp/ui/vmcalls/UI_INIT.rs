use super::super::MpUiExport;
use crate::boundary::generic::InboundVmCall;

/// `UI_INIT` MP UI exports vmMain boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:219`
pub struct UiInit;

impl InboundVmCall for UiInit {
    type Command = MpUiExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpUiExport = MpUiExport::UI_INIT;
}
