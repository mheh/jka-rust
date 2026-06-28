use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_SP_GETNUMLANGUAGES` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:135`
pub struct UiSpGetnumlanguages;

impl OutboundSysCall for UiSpGetnumlanguages {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_SP_GETNUMLANGUAGES;
}
