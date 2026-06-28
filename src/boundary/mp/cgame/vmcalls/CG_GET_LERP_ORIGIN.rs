use super::super::MpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_GET_LERP_ORIGIN` MP cgame exports vmMain boundary token.
///
/// Raven: int	CG_PointContents( const vec3_t point, int passEntityNum );
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:395`
pub struct CgGetLerpOrigin;

impl InboundVmCall for CgGetLerpOrigin {
    type Command = MpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpCgameExport = MpCgameExport::CG_GET_LERP_ORIGIN;
}
