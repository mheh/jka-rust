use super::super::MpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_G2MARK` MP cgame exports vmMain boundary token.
///
/// Raven: void CG_Trace( trace_t *result, const vec3_t start, const vec3_t mins, const vec3_t maxs, const vec3_t end,
/// Raven: int skipNumber, int mask );
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:410`
pub struct CgG2mark;

impl InboundVmCall for CgG2mark {
    type Command = MpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpCgameExport = MpCgameExport::CG_G2MARK;
}
