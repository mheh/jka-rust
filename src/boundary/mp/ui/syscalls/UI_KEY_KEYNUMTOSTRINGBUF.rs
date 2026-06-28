use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_KEY_KEYNUMTOSTRINGBUF` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:52`
pub struct UiKeyKeynumtostringbuf;

impl OutboundSysCall for UiKeyKeynumtostringbuf {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_KEY_KEYNUMTOSTRINGBUF;
}
