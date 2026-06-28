use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_CM_LERPTAG` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:181`
pub struct UiCmLerptag;

impl OutboundSysCall for UiCmLerptag {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_CM_LERPTAG;
}
