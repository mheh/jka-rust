use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_CVAR_REGISTER` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:69`
pub struct UiCvarRegister;

impl OutboundSysCall for UiCvarRegister {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_CVAR_REGISTER;
}
