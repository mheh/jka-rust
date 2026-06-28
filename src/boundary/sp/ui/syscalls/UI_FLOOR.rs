use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_FLOOR` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:246`
pub struct UiFloor;

impl OutboundSysCall for UiFloor {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_FLOOR;
}
