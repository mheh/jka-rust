use super::super::SpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_INIT` SP cgame exports vmMain boundary token.
///
/// Source: `oracle/oracle/code/client/vmachine.h:14`
pub struct CgInit;

impl InboundVmCall for CgInit {
    type Command = SpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: SpCgameExport = SpCgameExport::CG_INIT;
}
