use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_LANGUAGE_ISASIAN` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:80`
pub struct UiLanguageIsasian;

impl OutboundSysCall for UiLanguageIsasian {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_LANGUAGE_ISASIAN;
}
