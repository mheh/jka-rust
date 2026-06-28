use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_G2_SETRAGDOLL` MP UI imports syscall boundary token.
///
/// Raven: rww - RAGDOLL_BEGIN
/// Source: `oracle/oracle/codemp/ui/ui_public.h:175`
pub struct UiG2Setragdoll;

impl OutboundSysCall for UiG2Setragdoll {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_G2_SETRAGDOLL;
}
