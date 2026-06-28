use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_PC_READ_TOKEN` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:87`
pub struct UiPcReadToken;

impl OutboundSysCall for UiPcReadToken {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_PC_READ_TOKEN;
}
