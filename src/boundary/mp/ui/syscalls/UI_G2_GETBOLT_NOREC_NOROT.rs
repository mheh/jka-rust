use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_G2_GETBOLT_NOREC_NOROT` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:148`
pub struct UiG2GetboltNorecNorot;

impl OutboundSysCall for UiG2GetboltNorecNorot {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_G2_GETBOLT_NOREC_NOROT;
}
