use super::super::SpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_RESIZE_G2_SURFACE` SP cgame exports vmMain boundary token.
///
/// Source: `oracle/oracle/code/client/vmachine.h:28`
pub struct CgResizeG2Surface;

impl InboundVmCall for CgResizeG2Surface {
    type Command = SpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: SpCgameExport = SpCgameExport::CG_RESIZE_G2_SURFACE;
}
