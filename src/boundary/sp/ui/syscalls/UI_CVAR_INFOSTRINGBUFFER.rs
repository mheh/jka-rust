use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_CVAR_INFOSTRINGBUFFER` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:161`
pub struct UiCvarInfostringbuffer;

impl OutboundSysCall for UiCvarInfostringbuffer {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_CVAR_INFOSTRINGBUFFER;
}
