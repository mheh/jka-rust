use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_LAN_COMPARESERVERS` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:114`
pub struct UiLanCompareservers;

impl OutboundSysCall for UiLanCompareservers {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_LAN_COMPARESERVERS;
}
