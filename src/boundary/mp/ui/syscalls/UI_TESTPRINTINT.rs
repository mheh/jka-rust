use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_TESTPRINTINT` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:129`
pub struct UiTestprintint;

impl OutboundSysCall for UiTestprintint {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_TESTPRINTINT;
}
