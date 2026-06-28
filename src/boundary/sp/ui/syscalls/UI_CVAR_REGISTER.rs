use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_CVAR_REGISTER` SP UI imports syscall boundary token.
///
/// Raven: 50
/// Source: `oracle/oracle/code/ui/ui_public.h:202`
pub struct UiCvarRegister;

impl OutboundSysCall for UiCvarRegister {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_CVAR_REGISTER;
}
