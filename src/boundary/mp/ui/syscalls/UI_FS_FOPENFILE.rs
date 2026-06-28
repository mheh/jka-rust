use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_FS_FOPENFILE` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:31`
pub struct UiFsFopenfile;

impl OutboundSysCall for UiFsFopenfile {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_FS_FOPENFILE;
}
