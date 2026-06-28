use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_MILLISECONDS` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:154`
pub struct UiMilliseconds;

impl OutboundSysCall for UiMilliseconds {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_MILLISECONDS;
}
