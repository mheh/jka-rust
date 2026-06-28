use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_SET_CDKEY` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:73`
pub struct UiSetCdkey;

impl OutboundSysCall for UiSetCdkey {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_SET_CDKEY;
}
