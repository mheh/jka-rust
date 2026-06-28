use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_CIN_SETEXTENTS` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:109`
pub struct UiCinSetextents;

impl OutboundSysCall for UiCinSetextents {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_CIN_SETEXTENTS;
}
