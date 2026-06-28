use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_LAN_LOADCACHEDSERVERS` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:223`
pub struct UiLanLoadcachedservers;

impl OutboundSysCall for UiLanLoadcachedservers {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_LAN_LOADCACHEDSERVERS;
}
