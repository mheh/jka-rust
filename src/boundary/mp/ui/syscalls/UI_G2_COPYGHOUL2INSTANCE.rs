use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_G2_COPYGHOUL2INSTANCE` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:158`
pub struct UiG2Copyghoul2instance;

impl OutboundSysCall for UiG2Copyghoul2instance {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_G2_COPYGHOUL2INSTANCE;
}
