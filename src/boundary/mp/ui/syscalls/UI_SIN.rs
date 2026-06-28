use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_SIN` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:119`
pub struct UiSin;

impl OutboundSysCall for UiSin {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_SIN;
}
