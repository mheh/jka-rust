use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_STRNCPY` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:241`
pub struct UiStrncpy;

impl OutboundSysCall for UiStrncpy {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_STRNCPY;
}
