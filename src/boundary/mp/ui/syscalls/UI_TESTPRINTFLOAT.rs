use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_TESTPRINTFLOAT` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:130`
pub struct UiTestprintfloat;

impl OutboundSysCall for UiTestprintfloat {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_TESTPRINTFLOAT;
}
