use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_CM_LERPTAG` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:48`
pub struct UiCmLerptag;

impl OutboundSysCall for UiCmLerptag {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_CM_LERPTAG;
}
