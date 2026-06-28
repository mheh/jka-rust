use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_FS_GETFILELIST` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:35`
pub struct UiFsGetfilelist;

impl OutboundSysCall for UiFsGetfilelist {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_FS_GETFILELIST;
}
