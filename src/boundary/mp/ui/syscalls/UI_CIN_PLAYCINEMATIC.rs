use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_CIN_PLAYCINEMATIC` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:105`
pub struct UiCinPlaycinematic;

impl OutboundSysCall for UiCinPlaycinematic {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_CIN_PLAYCINEMATIC;
}
