use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CGAME_MATRIXMULTIPLY` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:137`
pub struct CgameMatrixmultiply;

impl OutboundSysCall for CgameMatrixmultiply {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CGAME_MATRIXMULTIPLY;
}
