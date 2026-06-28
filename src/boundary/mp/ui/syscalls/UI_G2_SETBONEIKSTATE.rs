use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_G2_SETBONEIKSTATE` MP UI imports syscall boundary token.
///
/// Raven: rww - RAGDOLL_END
/// Raven: rww - ik move method, allows you to specify a bone and move it to a world point (within joint constraints)
/// Raven: by using the majority of gil's existing bone angling stuff from the ragdoll code.
/// Source: `oracle/oracle/codemp/ui/ui_public.h:183`
pub struct UiG2Setboneikstate;

impl OutboundSysCall for UiG2Setboneikstate {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_G2_SETBONEIKSTATE;
}
