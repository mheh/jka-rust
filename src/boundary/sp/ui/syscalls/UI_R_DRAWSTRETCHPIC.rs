use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_R_DRAWSTRETCHPIC` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:179`
pub struct UiRDrawstretchpic;

impl OutboundSysCall for UiRDrawstretchpic {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_R_DRAWSTRETCHPIC;
}
