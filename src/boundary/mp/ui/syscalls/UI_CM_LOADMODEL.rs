use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_CM_LOADMODEL` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:49`
pub struct UiCmLoadmodel;

impl OutboundSysCall for UiCmLoadmodel {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_CM_LOADMODEL;
}
