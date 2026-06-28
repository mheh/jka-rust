use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_S_STARTBACKGROUNDTRACK` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:215`
pub struct UiSStartbackgroundtrack;

impl OutboundSysCall for UiSStartbackgroundtrack {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_S_STARTBACKGROUNDTRACK;
}
