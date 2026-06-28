use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_ERROR` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:152`
pub struct UiError;

impl OutboundSysCall for UiError {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_ERROR;
}
