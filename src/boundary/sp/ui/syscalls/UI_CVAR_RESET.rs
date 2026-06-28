use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_CVAR_RESET` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:159`
pub struct UiCvarReset;

impl OutboundSysCall for UiCvarReset {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_CVAR_RESET;
}
