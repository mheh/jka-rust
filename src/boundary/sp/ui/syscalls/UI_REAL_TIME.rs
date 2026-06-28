use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_REAL_TIME` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:216`
pub struct UiRealTime;

impl OutboundSysCall for UiRealTime {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_REAL_TIME;
}
