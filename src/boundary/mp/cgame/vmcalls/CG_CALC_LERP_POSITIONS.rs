use super::super::MpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_CALC_LERP_POSITIONS` MP cgame exports vmMain boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:402`
pub struct CgCalcLerpPositions;

impl InboundVmCall for CgCalcLerpPositions {
    type Command = MpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpCgameExport = MpCgameExport::CG_CALC_LERP_POSITIONS;
}
