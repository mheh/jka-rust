use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_PC_FREE_SOURCE` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:86`
pub struct UiPcFreeSource;

impl OutboundSysCall for UiPcFreeSource {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_PC_FREE_SOURCE;
}
