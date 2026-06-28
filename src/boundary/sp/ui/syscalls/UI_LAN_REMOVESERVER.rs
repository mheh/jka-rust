use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_LAN_REMOVESERVER` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:226`
pub struct UiLanRemoveserver;

impl OutboundSysCall for UiLanRemoveserver {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_LAN_REMOVESERVER;
}
