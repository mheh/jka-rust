use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_ATAN2` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:244`
pub struct UiAtan2;

impl OutboundSysCall for UiAtan2 {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_ATAN2;
}
