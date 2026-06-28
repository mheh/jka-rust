use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_PC_READ_TOKEN` SP UI imports syscall boundary token.
///
/// Raven: 60
/// Source: `oracle/oracle/code/ui/ui_public.h:212`
pub struct UiPcReadToken;

impl OutboundSysCall for UiPcReadToken {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_PC_READ_TOKEN;
}
