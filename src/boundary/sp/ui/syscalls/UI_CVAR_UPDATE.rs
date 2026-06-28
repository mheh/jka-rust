use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_CVAR_UPDATE` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:203`
pub struct UiCvarUpdate;

impl OutboundSysCall for UiCvarUpdate {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_CVAR_UPDATE;
}
