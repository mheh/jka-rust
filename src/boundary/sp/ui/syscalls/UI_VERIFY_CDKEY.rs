use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_VERIFY_CDKEY` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:233`
pub struct UiVerifyCdkey;

impl OutboundSysCall for UiVerifyCdkey {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_VERIFY_CDKEY;
}
