use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_S_STOPBACKGROUNDTRACK` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:92`
pub struct UiSStopbackgroundtrack;

impl OutboundSysCall for UiSStopbackgroundtrack {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_S_STOPBACKGROUNDTRACK;
}
