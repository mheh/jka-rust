use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_ARGC` SP UI imports syscall boundary token.
///
/// Raven: 10
/// Source: `oracle/oracle/code/ui/ui_public.h:162`
pub struct UiArgc;

impl OutboundSysCall for UiArgc {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_ARGC;
}
