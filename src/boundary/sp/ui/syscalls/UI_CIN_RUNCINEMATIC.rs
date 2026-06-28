use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_CIN_RUNCINEMATIC` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:229`
pub struct UiCinRuncinematic;

impl OutboundSysCall for UiCinRuncinematic {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_CIN_RUNCINEMATIC;
}
