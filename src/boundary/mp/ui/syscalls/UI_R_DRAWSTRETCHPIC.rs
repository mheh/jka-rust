use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_R_DRAWSTRETCHPIC` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:46`
pub struct UiRDrawstretchpic;

impl OutboundSysCall for UiRDrawstretchpic {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_R_DRAWSTRETCHPIC;
}
