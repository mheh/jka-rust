use super::super::MpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_INCOMING_CONSOLE_COMMAND` MP cgame exports vmMain boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:414`
pub struct CgIncomingConsoleCommand;

impl InboundVmCall for CgIncomingConsoleCommand {
    type Command = MpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpCgameExport = MpCgameExport::CG_INCOMING_CONSOLE_COMMAND;
}
