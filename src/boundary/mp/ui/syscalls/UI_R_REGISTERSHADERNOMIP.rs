use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_R_REGISTERSHADERNOMIP` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:38`
pub struct UiRRegistershadernomip;

impl OutboundSysCall for UiRRegistershadernomip {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_R_REGISTERSHADERNOMIP;
}
