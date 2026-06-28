use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_KEY_GETBINDINGBUF` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:186`
pub struct UiKeyGetbindingbuf;

impl OutboundSysCall for UiKeyGetbindingbuf {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_KEY_GETBINDINGBUF;
}
