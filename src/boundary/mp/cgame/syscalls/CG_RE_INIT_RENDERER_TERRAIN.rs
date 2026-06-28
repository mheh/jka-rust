use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_RE_INIT_RENDERER_TERRAIN` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:332`
pub struct CgReInitRendererTerrain;

impl OutboundSysCall for CgReInitRendererTerrain {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_RE_INIT_RENDERER_TERRAIN;
}
