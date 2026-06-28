use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_LAN_GETPING` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:200`
pub struct UiLanGetping;

impl OutboundSysCall for UiLanGetping {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_LAN_GETPING;
}
