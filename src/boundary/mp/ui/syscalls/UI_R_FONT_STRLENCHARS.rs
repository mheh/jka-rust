use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_R_FONT_STRLENCHARS` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:77`
pub struct UiRFontStrlenchars;

impl OutboundSysCall for UiRFontStrlenchars {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_R_FONT_STRLENCHARS;
}
