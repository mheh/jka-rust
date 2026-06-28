use super::super::MpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_GET_ANGLES` MP cgame exports vmMain boundary token.
///
/// Raven: int entnum, vec3_t angle
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:419`
pub struct CgGetAngles;

impl InboundVmCall for CgGetAngles {
    type Command = MpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpCgameExport = MpCgameExport::CG_GET_ANGLES;
}
