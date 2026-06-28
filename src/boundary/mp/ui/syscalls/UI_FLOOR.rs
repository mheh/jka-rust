use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_FLOOR` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:126`
pub struct UiFloor;

impl OutboundSysCall for UiFloor {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_FLOOR;
}
