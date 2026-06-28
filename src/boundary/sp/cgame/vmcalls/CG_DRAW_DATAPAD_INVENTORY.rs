use super::super::SpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_DRAW_DATAPAD_INVENTORY` SP cgame exports vmMain boundary token.
///
/// Source: `oracle/oracle/code/client/vmachine.h:36`
pub struct CgDrawDatapadInventory;

impl InboundVmCall for CgDrawDatapadInventory {
    type Command = SpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: SpCgameExport = SpCgameExport::CG_DRAW_DATAPAD_INVENTORY;
}
