use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_ATAN2` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:121`
pub struct UiAtan2;

impl OutboundSysCall for UiAtan2 {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_ATAN2;
}
