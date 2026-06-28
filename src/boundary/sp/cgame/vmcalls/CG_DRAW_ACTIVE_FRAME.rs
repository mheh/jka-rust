use super::super::SpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_DRAW_ACTIVE_FRAME` SP cgame exports vmMain boundary token.
///
/// Source: `oracle/oracle/code/client/vmachine.h:17`
pub struct CgDrawActiveFrame;

impl InboundVmCall for CgDrawActiveFrame {
    type Command = SpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: SpCgameExport = SpCgameExport::CG_DRAW_ACTIVE_FRAME;
}
