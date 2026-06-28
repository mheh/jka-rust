use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_PC_LOAD_SOURCE` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:210`
pub struct UiPcLoadSource;

impl OutboundSysCall for UiPcLoadSource {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_PC_LOAD_SOURCE;
}
