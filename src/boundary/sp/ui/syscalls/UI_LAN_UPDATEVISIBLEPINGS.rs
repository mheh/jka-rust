use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_LAN_UPDATEVISIBLEPINGS` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:221`
pub struct UiLanUpdatevisiblepings;

impl OutboundSysCall for UiLanUpdatevisiblepings {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_LAN_UPDATEVISIBLEPINGS;
}
