use super::super::MpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_GET_ORIGIN_TRAJECTORY` MP cgame exports vmMain boundary token.
///
/// Raven: int entnum
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:421`
pub struct CgGetOriginTrajectory;

impl InboundVmCall for CgGetOriginTrajectory {
    type Command = MpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpCgameExport = MpCgameExport::CG_GET_ORIGIN_TRAJECTORY;
}
