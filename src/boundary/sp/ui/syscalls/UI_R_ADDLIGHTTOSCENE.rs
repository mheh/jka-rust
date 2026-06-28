use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_R_ADDLIGHTTOSCENE` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:176`
pub struct UiRAddlighttoscene;

impl OutboundSysCall for UiRAddlighttoscene {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_R_ADDLIGHTTOSCENE;
}
