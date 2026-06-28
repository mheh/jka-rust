use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_PRINT` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:153`
pub struct UiPrint;

impl OutboundSysCall for UiPrint {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_PRINT;
}
