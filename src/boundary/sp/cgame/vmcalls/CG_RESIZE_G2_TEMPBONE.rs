use super::super::SpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_RESIZE_G2_TEMPBONE` SP cgame exports vmMain boundary token.
///
/// Source: `oracle/oracle/code/client/vmachine.h:29`
pub struct CgResizeG2Tempbone;

impl InboundVmCall for CgResizeG2Tempbone {
    type Command = SpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: SpCgameExport = SpCgameExport::CG_RESIZE_G2_TEMPBONE;
}
