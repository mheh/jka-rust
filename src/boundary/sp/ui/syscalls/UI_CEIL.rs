use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_CEIL` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:247`
pub struct UiCeil;

impl OutboundSysCall for UiCeil {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_CEIL;
}
