use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_GETCLIENTSTATE` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:63`
pub struct UiGetclientstate;

impl OutboundSysCall for UiGetclientstate {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_GETCLIENTSTATE;
}
