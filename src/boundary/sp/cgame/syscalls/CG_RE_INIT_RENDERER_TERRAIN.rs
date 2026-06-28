use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_RE_INIT_RENDERER_TERRAIN` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:80`
pub struct CgReInitRendererTerrain;

impl OutboundSysCall for CgReInitRendererTerrain {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_RE_INIT_RENDERER_TERRAIN;
}
