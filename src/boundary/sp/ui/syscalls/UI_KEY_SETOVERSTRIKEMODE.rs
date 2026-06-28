use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_KEY_SETOVERSTRIKEMODE` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:190`
pub struct UiKeySetoverstrikemode;

impl OutboundSysCall for UiKeySetoverstrikemode {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_KEY_SETOVERSTRIKEMODE;
}
