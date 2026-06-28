use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_UPDATESCREEN` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:180`
pub struct UiUpdatescreen;

impl OutboundSysCall for UiUpdatescreen {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_UPDATESCREEN;
}
