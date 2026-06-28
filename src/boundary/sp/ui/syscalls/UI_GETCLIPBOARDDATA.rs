use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_GETCLIPBOARDDATA` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:194`
pub struct UiGetclipboarddata;

impl OutboundSysCall for UiGetclipboarddata {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_GETCLIPBOARDDATA;
}
