use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_FX_ADD_SCHEDULED_EFFECTS` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:226`
pub struct CgFxAddScheduledEffects;

impl OutboundSysCall for CgFxAddScheduledEffects {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_FX_ADD_SCHEDULED_EFFECTS;
}
