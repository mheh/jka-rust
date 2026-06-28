use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_KEY_SETCATCHER` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:193`
pub struct UiKeySetcatcher;

impl OutboundSysCall for UiKeySetcatcher {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_KEY_SETCATCHER;
}
