use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_KEY_SETBINDING` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:54`
pub struct UiKeySetbinding;

impl OutboundSysCall for UiKeySetbinding {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_KEY_SETBINDING;
}
