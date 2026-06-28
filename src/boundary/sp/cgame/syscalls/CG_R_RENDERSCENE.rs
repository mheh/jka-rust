use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_RENDERSCENE` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:139`
pub struct CgRRenderscene;

impl OutboundSysCall for CgRRenderscene {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_RENDERSCENE;
}
