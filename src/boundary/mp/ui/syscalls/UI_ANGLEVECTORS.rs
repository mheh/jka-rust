use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_ANGLEVECTORS` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:124`
pub struct UiAnglevectors;

impl OutboundSysCall for UiAnglevectors {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_ANGLEVECTORS;
}
