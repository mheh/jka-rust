use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_CVAR_CREATE` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:160`
pub struct UiCvarCreate;

impl OutboundSysCall for UiCvarCreate {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_CVAR_CREATE;
}
