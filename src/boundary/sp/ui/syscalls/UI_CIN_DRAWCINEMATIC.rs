use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_CIN_DRAWCINEMATIC` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:230`
pub struct UiCinDrawcinematic;

impl OutboundSysCall for UiCinDrawcinematic {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_CIN_DRAWCINEMATIC;
}
