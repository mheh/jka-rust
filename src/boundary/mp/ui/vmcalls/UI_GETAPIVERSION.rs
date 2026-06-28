use super::super::MpUiExport;
use crate::boundary::generic::InboundVmCall;

/// `UI_GETAPIVERSION` MP UI exports vmMain boundary token.
///
/// Raven: system reserved
/// Source: `oracle/oracle/codemp/ui/ui_public.h:217`
pub struct UiGetapiversion;

impl InboundVmCall for UiGetapiversion {
    type Command = MpUiExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpUiExport = MpUiExport::UI_GETAPIVERSION;
}
