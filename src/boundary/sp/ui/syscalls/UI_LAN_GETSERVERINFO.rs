use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_LAN_GETSERVERINFO` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:219`
pub struct UiLanGetserverinfo;

impl OutboundSysCall for UiLanGetserverinfo {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_LAN_GETSERVERINFO;
}
