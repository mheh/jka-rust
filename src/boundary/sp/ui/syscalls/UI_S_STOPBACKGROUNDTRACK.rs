use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_S_STOPBACKGROUNDTRACK` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:214`
pub struct UiSStopbackgroundtrack;

impl OutboundSysCall for UiSStopbackgroundtrack {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_S_STOPBACKGROUNDTRACK;
}
