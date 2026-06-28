use super::super::SpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_SHUTDOWN` SP cgame exports vmMain boundary token.
///
/// Source: `oracle/oracle/code/client/vmachine.h:15`
pub struct CgShutdown;

impl InboundVmCall for CgShutdown {
    type Command = SpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: SpCgameExport = SpCgameExport::CG_SHUTDOWN;
}
