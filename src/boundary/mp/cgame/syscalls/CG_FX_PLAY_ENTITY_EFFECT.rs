use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_FX_PLAY_ENTITY_EFFECT` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:221`
pub struct CgFxPlayEntityEffect;

impl OutboundSysCall for CgFxPlayEntityEffect {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_FX_PLAY_ENTITY_EFFECT;
}
