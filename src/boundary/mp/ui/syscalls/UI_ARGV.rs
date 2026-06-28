use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_ARGV` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:29`
pub struct UiArgv;

impl OutboundSysCall for UiArgv {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_ARGV;
}
