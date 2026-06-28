use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_G2_DUPLICATEGHOUL2INSTANCE` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:160`
pub struct UiG2Duplicateghoul2instance;

impl OutboundSysCall for UiG2Duplicateghoul2instance {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_G2_DUPLICATEGHOUL2INSTANCE;
}
