use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_COS` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:120`
pub struct UiCos;

impl OutboundSysCall for UiCos {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_COS;
}
