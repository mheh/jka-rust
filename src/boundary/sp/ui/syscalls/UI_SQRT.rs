use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_SQRT` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:245`
pub struct UiSqrt;

impl OutboundSysCall for UiSqrt {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_SQRT;
}
