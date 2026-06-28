use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_MEMSET` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:239`
pub struct UiMemset;

impl OutboundSysCall for UiMemset {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_MEMSET;
}
