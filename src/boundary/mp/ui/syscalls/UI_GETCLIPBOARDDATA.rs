use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_GETCLIPBOARDDATA` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:61`
pub struct UiGetclipboarddata;

impl OutboundSysCall for UiGetclipboarddata {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_GETCLIPBOARDDATA;
}
