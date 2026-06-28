use super::super::SpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_RESIZE_G2_BOLT` SP cgame exports vmMain boundary token.
///
/// Raven: Ghoul2 Insert Start
/// Source: `oracle/oracle/code/client/vmachine.h:25`
pub struct CgResizeG2Bolt;

impl InboundVmCall for CgResizeG2Bolt {
    type Command = SpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: SpCgameExport = SpCgameExport::CG_RESIZE_G2_BOLT;
}
