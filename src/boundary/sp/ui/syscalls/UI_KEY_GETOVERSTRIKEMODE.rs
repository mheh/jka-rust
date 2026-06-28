use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_KEY_GETOVERSTRIKEMODE` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:189`
pub struct UiKeyGetoverstrikemode;

impl OutboundSysCall for UiKeyGetoverstrikemode {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_KEY_GETOVERSTRIKEMODE;
}
