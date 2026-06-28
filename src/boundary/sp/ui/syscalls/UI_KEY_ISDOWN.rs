use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_KEY_ISDOWN` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:188`
pub struct UiKeyIsdown;

impl OutboundSysCall for UiKeyIsdown {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_KEY_ISDOWN;
}
