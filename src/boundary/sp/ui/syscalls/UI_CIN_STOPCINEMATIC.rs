use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_CIN_STOPCINEMATIC` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:228`
pub struct UiCinStopcinematic;

impl OutboundSysCall for UiCinStopcinematic {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_CIN_STOPCINEMATIC;
}
