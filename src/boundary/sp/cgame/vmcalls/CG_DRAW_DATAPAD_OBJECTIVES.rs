use super::super::SpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_DRAW_DATAPAD_OBJECTIVES` SP cgame exports vmMain boundary token.
///
/// Source: `oracle/oracle/code/client/vmachine.h:34`
pub struct CgDrawDatapadObjectives;

impl InboundVmCall for CgDrawDatapadObjectives {
    type Command = SpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: SpCgameExport = SpCgameExport::CG_DRAW_DATAPAD_OBJECTIVES;
}
