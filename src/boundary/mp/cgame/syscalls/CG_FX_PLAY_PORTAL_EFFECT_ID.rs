use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_FX_PLAY_PORTAL_EFFECT_ID` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:223`
pub struct CgFxPlayPortalEffectId;

impl OutboundSysCall for CgFxPlayPortalEffectId {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_FX_PLAY_PORTAL_EFFECT_ID;
}
