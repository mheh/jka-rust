use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_GET_BMODEL_VERTS` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:182`
pub struct CgRGetBmodelVerts;

impl OutboundSysCall for CgRGetBmodelVerts {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_GET_BMODEL_VERTS;
}
