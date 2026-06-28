use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_FX_PLAY_BOLTED_EFFECT_ID` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:225`
pub struct CgFxPlayBoltedEffectId;

impl OutboundSysCall for CgFxPlayBoltedEffectId {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_FX_PLAY_BOLTED_EFFECT_ID;
}
