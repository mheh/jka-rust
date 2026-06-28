use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_FS_FCLOSEFILE` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:34`
pub struct UiFsFclosefile;

impl OutboundSysCall for UiFsFclosefile {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_FS_FCLOSEFILE;
}
