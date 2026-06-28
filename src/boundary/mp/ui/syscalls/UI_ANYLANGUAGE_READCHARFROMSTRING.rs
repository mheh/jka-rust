use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_ANYLANGUAGE_READCHARFROMSTRING` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:82`
pub struct UiAnylanguageReadcharfromstring;

impl OutboundSysCall for UiAnylanguageReadcharfromstring {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_ANYLANGUAGE_READCHARFROMSTRING;
}
