use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_SIN` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:242`
pub struct UiSin;

impl OutboundSysCall for UiSin {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_SIN;
}
