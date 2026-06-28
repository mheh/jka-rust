use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_R_REGISTERMODEL` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:170`
pub struct UiRRegistermodel;

impl OutboundSysCall for UiRRegistermodel {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_R_REGISTERMODEL;
}
