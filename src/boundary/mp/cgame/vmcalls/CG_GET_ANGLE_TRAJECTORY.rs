use super::super::MpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_GET_ANGLE_TRAJECTORY` MP cgame exports vmMain boundary token.
///
/// Raven: int entnum
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:422`
pub struct CgGetAngleTrajectory;

impl InboundVmCall for CgGetAngleTrajectory {
    type Command = MpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpCgameExport = MpCgameExport::CG_GET_ANGLE_TRAJECTORY;
}
