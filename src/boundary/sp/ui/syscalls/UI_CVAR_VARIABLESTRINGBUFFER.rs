use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_CVAR_VARIABLESTRINGBUFFER` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:157`
pub struct UiCvarVariablestringbuffer;

impl OutboundSysCall for UiCvarVariablestringbuffer {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_CVAR_VARIABLESTRINGBUFFER;
}
