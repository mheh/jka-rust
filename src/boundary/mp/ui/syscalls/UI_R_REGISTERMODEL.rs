use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_R_REGISTERMODEL` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:36`
pub struct UiRRegistermodel;

impl OutboundSysCall for UiRRegistermodel {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_R_REGISTERMODEL;
}
