use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_KEY_SETOVERSTRIKEMODE` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:57`
pub struct UiKeySetoverstrikemode;

impl OutboundSysCall for UiKeySetoverstrikemode {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_KEY_SETOVERSTRIKEMODE;
}
