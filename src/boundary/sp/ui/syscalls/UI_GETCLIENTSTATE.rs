use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_GETCLIENTSTATE` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:196`
pub struct UiGetclientstate;

impl OutboundSysCall for UiGetclientstate {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_GETCLIENTSTATE;
}
