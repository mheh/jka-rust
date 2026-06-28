use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_R_ADDLIGHTTOSCENE` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:43`
pub struct UiRAddlighttoscene;

impl OutboundSysCall for UiRAddlighttoscene {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_R_ADDLIGHTTOSCENE;
}
