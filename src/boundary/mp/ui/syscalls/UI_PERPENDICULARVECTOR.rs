use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_PERPENDICULARVECTOR` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:125`
pub struct UiPerpendicularvector;

impl OutboundSysCall for UiPerpendicularvector {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_PERPENDICULARVECTOR;
}
