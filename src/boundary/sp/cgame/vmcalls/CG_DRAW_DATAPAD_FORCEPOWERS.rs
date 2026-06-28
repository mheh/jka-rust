use super::super::SpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_DRAW_DATAPAD_FORCEPOWERS` SP cgame exports vmMain boundary token.
///
/// Source: `oracle/oracle/code/client/vmachine.h:37`
pub struct CgDrawDatapadForcepowers;

impl InboundVmCall for CgDrawDatapadForcepowers {
    type Command = SpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: SpCgameExport = SpCgameExport::CG_DRAW_DATAPAD_FORCEPOWERS;
}
