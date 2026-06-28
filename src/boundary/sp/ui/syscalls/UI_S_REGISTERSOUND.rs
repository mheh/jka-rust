use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_S_REGISTERSOUND` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:183`
pub struct UiSRegistersound;

impl OutboundSysCall for UiSRegistersound {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_S_REGISTERSOUND;
}
