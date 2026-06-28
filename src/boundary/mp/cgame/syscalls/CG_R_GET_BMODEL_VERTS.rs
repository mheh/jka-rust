use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_GET_BMODEL_VERTS` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:170`
pub struct CgRGetBmodelVerts;

impl OutboundSysCall for CgRGetBmodelVerts {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_GET_BMODEL_VERTS;
}
