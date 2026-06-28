use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_R_REGISTERFONT` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:207`
pub struct UiRRegisterfont;

impl OutboundSysCall for UiRRegisterfont {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_R_REGISTERFONT;
}
