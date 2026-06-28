use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_R_ADDREFENTITYTOSCENE` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:174`
pub struct UiRAddrefentitytoscene;

impl OutboundSysCall for UiRAddrefentitytoscene {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_R_ADDREFENTITYTOSCENE;
}
