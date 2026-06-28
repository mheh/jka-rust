use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_CIN_PLAYCINEMATIC` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:227`
pub struct UiCinPlaycinematic;

impl OutboundSysCall for UiCinPlaycinematic {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_CIN_PLAYCINEMATIC;
}
