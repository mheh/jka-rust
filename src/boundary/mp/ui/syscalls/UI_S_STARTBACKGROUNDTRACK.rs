use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_S_STARTBACKGROUNDTRACK` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:93`
pub struct UiSStartbackgroundtrack;

impl OutboundSysCall for UiSStartbackgroundtrack {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_S_STARTBACKGROUNDTRACK;
}
