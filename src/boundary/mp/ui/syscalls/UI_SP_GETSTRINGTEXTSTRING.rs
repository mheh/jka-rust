use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_SP_GETSTRINGTEXTSTRING` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:137`
pub struct UiSpGetstringtextstring;

impl OutboundSysCall for UiSpGetstringtextstring {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_SP_GETSTRINGTEXTSTRING;
}
