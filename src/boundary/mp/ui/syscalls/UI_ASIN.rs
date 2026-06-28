use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_ASIN` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:133`
pub struct UiAsin;

impl OutboundSysCall for UiAsin {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_ASIN;
}
