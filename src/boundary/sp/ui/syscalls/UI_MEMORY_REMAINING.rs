use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_MEMORY_REMAINING` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:204`
pub struct UiMemoryRemaining;

impl OutboundSysCall for UiMemoryRemaining {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_MEMORY_REMAINING;
}
