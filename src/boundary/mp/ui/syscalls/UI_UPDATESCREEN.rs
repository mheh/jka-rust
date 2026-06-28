use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_UPDATESCREEN` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:47`
pub struct UiUpdatescreen;

impl OutboundSysCall for UiUpdatescreen {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_UPDATESCREEN;
}
