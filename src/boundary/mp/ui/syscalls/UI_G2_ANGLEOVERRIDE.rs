use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_G2_ANGLEOVERRIDE` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:153`
pub struct UiG2Angleoverride;

impl OutboundSysCall for UiG2Angleoverride {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_G2_ANGLEOVERRIDE;
}
