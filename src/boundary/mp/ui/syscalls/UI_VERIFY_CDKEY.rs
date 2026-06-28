use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_VERIFY_CDKEY` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:74`
pub struct UiVerifyCdkey;

impl OutboundSysCall for UiVerifyCdkey {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_VERIFY_CDKEY;
}
