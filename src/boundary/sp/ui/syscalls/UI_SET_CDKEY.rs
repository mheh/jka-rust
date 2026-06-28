use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_SET_CDKEY` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:206`
pub struct UiSetCdkey;

impl OutboundSysCall for UiSetCdkey {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_SET_CDKEY;
}
