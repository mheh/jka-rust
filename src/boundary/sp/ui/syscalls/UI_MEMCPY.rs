use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_MEMCPY` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:240`
pub struct UiMemcpy;

impl OutboundSysCall for UiMemcpy {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_MEMCPY;
}
