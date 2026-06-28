use super::super::MpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_CONSOLE_COMMAND` MP cgame exports vmMain boundary token.
///
/// Raven: void (*CG_Shutdown)( void );
/// Raven: oportunity to flush and close any open files
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:366`
pub struct CgConsoleCommand;

impl InboundVmCall for CgConsoleCommand {
    type Command = MpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpCgameExport = MpCgameExport::CG_CONSOLE_COMMAND;
}
