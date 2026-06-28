use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_ERROR` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:18`
pub struct UiError;

impl OutboundSysCall for UiError {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_ERROR;
}
