use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_PC_FREE_SOURCE` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:211`
pub struct UiPcFreeSource;

impl OutboundSysCall for UiPcFreeSource {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_PC_FREE_SOURCE;
}
