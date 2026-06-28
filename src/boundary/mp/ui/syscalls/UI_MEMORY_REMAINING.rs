use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_MEMORY_REMAINING` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:71`
pub struct UiMemoryRemaining;

impl OutboundSysCall for UiMemoryRemaining {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_MEMORY_REMAINING;
}
