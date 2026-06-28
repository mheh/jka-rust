use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_CIN_SETEXTENTS` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:231`
pub struct UiCinSetextents;

impl OutboundSysCall for UiCinSetextents {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_CIN_SETEXTENTS;
}
