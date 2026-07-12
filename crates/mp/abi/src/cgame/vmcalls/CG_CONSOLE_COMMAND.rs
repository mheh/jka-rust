use mp_qshared::shared::qboolean;

use super::super::MpCgameExport;
use abi_transport::generic::{DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport};

/// `CG_CONSOLE_COMMAND` MP cgame exports vmMain ABI token.
///
/// Raven: qboolean (*CG_ConsoleCommand)( void );
/// Raven: a console command has been issued locally that is not recognized by the
/// Raven: main game system.
/// Raven: use Cmd_Argc() / Cmd_Argv() to read the command, return qfalse if the
/// Raven: command is not known to the game
/// Enum value source: `oracle/codemp/cgame/cg_public.h:366-371`
/// Args source: `oracle/codemp/cgame/cg_main.c:199-200`
/// Output source: `oracle/codemp/cgame/cg_main.c:199-200`
/// Transport/call-site source: `oracle/codemp/client/cl_cgame.cpp:1815-1820`
pub struct CgConsoleCommand;

impl InboundVmCall for CgConsoleCommand {
    type Command = MpCgameExport;
    type Args = ();
    type Output = qboolean;

    const COMMAND: MpCgameExport = MpCgameExport::CG_CONSOLE_COMMAND;
}

impl DecodeVmMain for CgConsoleCommand {
    fn decode_vm_main(_transport: VmMainTransport) -> Self::Args {}
}

impl EncodeVmMainReturn for CgConsoleCommand {
    fn encode_return(output: Self::Output) -> isize {
        output as isize
    }
}
