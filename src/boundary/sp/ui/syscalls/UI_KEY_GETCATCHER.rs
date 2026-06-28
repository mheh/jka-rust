use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_KEY_GETCATCHER` SP UI imports syscall boundary token.
///
/// Raven: 40
/// Source: `oracle/oracle/code/ui/ui_public.h:192`
pub struct UiKeyGetcatcher;

impl OutboundSysCall for UiKeyGetcatcher {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_KEY_GETCATCHER;
}
