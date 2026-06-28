use super::super::MpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_GET_LERP_DATA` MP cgame exports vmMain boundary token.
///
/// Raven: void CG_LerpOrigin(int num, vec3_t result);
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:398`
pub struct CgGetLerpData;

impl InboundVmCall for CgGetLerpData {
    type Command = MpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpCgameExport = MpCgameExport::CG_GET_LERP_DATA;
}
