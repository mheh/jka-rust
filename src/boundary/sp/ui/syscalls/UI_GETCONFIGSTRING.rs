use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_GETCONFIGSTRING` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:197`
pub struct UiGetconfigstring;

impl OutboundSysCall for UiGetconfigstring {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_GETCONFIGSTRING;
}
