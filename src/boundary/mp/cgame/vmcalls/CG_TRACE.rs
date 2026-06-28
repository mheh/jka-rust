use super::super::MpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_TRACE` MP cgame exports vmMain boundary token.
///
/// Raven: void CG_CalcEntityLerpPositions(int num);
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:405`
pub struct CgTrace;

impl InboundVmCall for CgTrace {
    type Command = MpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpCgameExport = MpCgameExport::CG_TRACE;
}
