use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_PC_SOURCE_FILE_AND_LINE` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:88`
pub struct UiPcSourceFileAndLine;

impl OutboundSysCall for UiPcSourceFileAndLine {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_PC_SOURCE_FILE_AND_LINE;
}
