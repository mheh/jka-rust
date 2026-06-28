use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_G2_GETBOLT_NOREC` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:147`
pub struct UiG2GetboltNorec;

impl OutboundSysCall for UiG2GetboltNorec {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_G2_GETBOLT_NOREC;
}
