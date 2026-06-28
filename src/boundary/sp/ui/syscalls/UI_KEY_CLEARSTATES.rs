use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_KEY_CLEARSTATES` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:191`
pub struct UiKeyClearstates;

impl OutboundSysCall for UiKeyClearstates {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_KEY_CLEARSTATES;
}
