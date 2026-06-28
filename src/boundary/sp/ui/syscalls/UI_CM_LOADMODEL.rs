use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_CM_LOADMODEL` SP UI imports syscall boundary token.
///
/// Raven: 30
/// Source: `oracle/oracle/code/ui/ui_public.h:182`
pub struct UiCmLoadmodel;

impl OutboundSysCall for UiCmLoadmodel {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_CM_LOADMODEL;
}
