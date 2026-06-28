use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_CIN_STOPCINEMATIC` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:106`
pub struct UiCinStopcinematic;

impl OutboundSysCall for UiCinStopcinematic {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_CIN_STOPCINEMATIC;
}
