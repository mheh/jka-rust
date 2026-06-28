use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_CEIL` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:127`
pub struct UiCeil;

impl OutboundSysCall for UiCeil {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_CEIL;
}
