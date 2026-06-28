use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_GETGLCONFIG` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:62`
pub struct UiGetglconfig;

impl OutboundSysCall for UiGetglconfig {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_GETGLCONFIG;
}
