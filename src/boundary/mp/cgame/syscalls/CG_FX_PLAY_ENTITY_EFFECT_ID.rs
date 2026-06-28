use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_FX_PLAY_ENTITY_EFFECT_ID` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:224`
pub struct CgFxPlayEntityEffectId;

impl OutboundSysCall for CgFxPlayEntityEffectId {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_FX_PLAY_ENTITY_EFFECT_ID;
}
