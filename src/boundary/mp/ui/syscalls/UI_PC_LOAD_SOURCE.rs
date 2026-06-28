use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_PC_LOAD_SOURCE` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:85`
pub struct UiPcLoadSource;

impl OutboundSysCall for UiPcLoadSource {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_PC_LOAD_SOURCE;
}
