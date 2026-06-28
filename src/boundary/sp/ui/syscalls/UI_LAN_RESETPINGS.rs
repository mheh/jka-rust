use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_LAN_RESETPINGS` SP UI imports syscall boundary token.
///
/// Raven: 70
/// Source: `oracle/oracle/code/ui/ui_public.h:222`
pub struct UiLanResetpings;

impl OutboundSysCall for UiLanResetpings {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_LAN_RESETPINGS;
}
