use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_CVAR_UPDATE` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:70`
pub struct UiCvarUpdate;

impl OutboundSysCall for UiCvarUpdate {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_CVAR_UPDATE;
}
