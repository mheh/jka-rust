use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_FS_FCLOSEFILE` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:168`
pub struct UiFsFclosefile;

impl OutboundSysCall for UiFsFclosefile {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_FS_FCLOSEFILE;
}
