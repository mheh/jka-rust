use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_PC_SOURCE_FILE_AND_LINE` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:213`
pub struct UiPcSourceFileAndLine;

impl OutboundSysCall for UiPcSourceFileAndLine {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_PC_SOURCE_FILE_AND_LINE;
}
