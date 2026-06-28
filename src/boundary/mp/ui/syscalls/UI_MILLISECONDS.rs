use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_MILLISECONDS` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:20`
pub struct UiMilliseconds;

impl OutboundSysCall for UiMilliseconds {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_MILLISECONDS;
}
