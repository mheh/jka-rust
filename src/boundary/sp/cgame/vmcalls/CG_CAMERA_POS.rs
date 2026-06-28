use super::super::SpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_CAMERA_POS` SP cgame exports vmMain boundary token.
///
/// Source: `oracle/oracle/code/client/vmachine.h:19`
pub struct CgCameraPos;

impl InboundVmCall for CgCameraPos {
    type Command = SpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: SpCgameExport = SpCgameExport::CG_CAMERA_POS;
}
