use super::super::SpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_CONSOLE_COMMAND` SP cgame exports vmMain boundary token.
///
/// Source: `oracle/oracle/code/client/vmachine.h:16`
pub struct CgConsoleCommand;

impl InboundVmCall for CgConsoleCommand {
    type Command = SpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: SpCgameExport = SpCgameExport::CG_CONSOLE_COMMAND;
}
