use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_STRNCPY` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:118`
pub struct UiStrncpy;

impl OutboundSysCall for UiStrncpy {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_STRNCPY;
}
