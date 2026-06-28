use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_R_REGISTERSHADERNOMIP` SP UI imports syscall boundary token.
///
/// Raven: 20
/// Source: `oracle/oracle/code/ui/ui_public.h:172`
pub struct UiRRegistershadernomip;

impl OutboundSysCall for UiRRegistershadernomip {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_R_REGISTERSHADERNOMIP;
}
