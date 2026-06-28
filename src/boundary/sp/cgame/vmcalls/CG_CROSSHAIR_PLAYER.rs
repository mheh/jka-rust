use super::super::SpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_CROSSHAIR_PLAYER` SP cgame exports vmMain boundary token.
///
/// Source: `oracle/oracle/code/client/vmachine.h:18`
pub struct CgCrosshairPlayer;

impl InboundVmCall for CgCrosshairPlayer {
    type Command = SpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: SpCgameExport = SpCgameExport::CG_CROSSHAIR_PLAYER;
}
