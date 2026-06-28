use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_FX_DRAW_2D_EFFECTS` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:231`
pub struct CgFxDraw2dEffects;

impl OutboundSysCall for CgFxDraw2dEffects {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_FX_DRAW_2D_EFFECTS;
}
