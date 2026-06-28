use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_SQRT` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:122`
pub struct UiSqrt;

impl OutboundSysCall for UiSqrt {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_SQRT;
}
