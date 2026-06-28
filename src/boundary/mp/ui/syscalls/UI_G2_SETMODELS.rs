use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_G2_SETMODELS` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:144`
pub struct UiG2Setmodels;

impl OutboundSysCall for UiG2Setmodels {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_G2_SETMODELS;
}
