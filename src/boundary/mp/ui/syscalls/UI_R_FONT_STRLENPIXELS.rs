use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_R_FONT_STRLENPIXELS` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:76`
pub struct UiRFontStrlenpixels;

impl OutboundSysCall for UiRFontStrlenpixels {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_R_FONT_STRLENPIXELS;
}
