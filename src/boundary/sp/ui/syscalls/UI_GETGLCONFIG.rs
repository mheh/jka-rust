use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_GETGLCONFIG` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:195`
pub struct UiGetglconfig;

impl OutboundSysCall for UiGetglconfig {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_GETGLCONFIG;
}
