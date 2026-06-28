use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_FS_WRITE` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:167`
pub struct UiFsWrite;

impl OutboundSysCall for UiFsWrite {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_FS_WRITE;
}
