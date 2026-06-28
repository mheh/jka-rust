use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_LAN_SERVERSTATUS` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:111`
pub struct UiLanServerstatus;

impl OutboundSysCall for UiLanServerstatus {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_LAN_SERVERSTATUS;
}
