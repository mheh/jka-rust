use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_GET_CDKEY` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:72`
pub struct UiGetCdkey;

impl OutboundSysCall for UiGetCdkey {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_GET_CDKEY;
}
