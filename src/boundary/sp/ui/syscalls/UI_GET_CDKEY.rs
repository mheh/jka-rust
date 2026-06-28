use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_GET_CDKEY` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:205`
pub struct UiGetCdkey;

impl OutboundSysCall for UiGetCdkey {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_GET_CDKEY;
}
