use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_FS_GETFILELIST` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:169`
pub struct UiFsGetfilelist;

impl OutboundSysCall for UiFsGetfilelist {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_FS_GETFILELIST;
}
