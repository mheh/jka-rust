use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_ARGV` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:163`
pub struct UiArgv;

impl OutboundSysCall for UiArgv {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_ARGV;
}
