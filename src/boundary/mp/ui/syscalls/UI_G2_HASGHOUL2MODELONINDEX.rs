use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_G2_HASGHOUL2MODELONINDEX` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:161`
pub struct UiG2Hasghoul2modelonindex;

impl OutboundSysCall for UiG2Hasghoul2modelonindex {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_G2_HASGHOUL2MODELONINDEX;
}
