use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_R_FONT_STRHEIGHTPIXELS` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:78`
pub struct UiRFontStrheightpixels;

impl OutboundSysCall for UiRFontStrheightpixels {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_R_FONT_STRHEIGHTPIXELS;
}
