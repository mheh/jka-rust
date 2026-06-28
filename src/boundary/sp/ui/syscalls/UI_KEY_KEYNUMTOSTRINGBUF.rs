use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_KEY_KEYNUMTOSTRINGBUF` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:185`
pub struct UiKeyKeynumtostringbuf;

impl OutboundSysCall for UiKeyKeynumtostringbuf {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_KEY_KEYNUMTOSTRINGBUF;
}
