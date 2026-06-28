use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_GETCONFIGSTRING` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:64`
pub struct UiGetconfigstring;

impl OutboundSysCall for UiGetconfigstring {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_GETCONFIGSTRING;
}
