use super::super::MpGameExport;
use mp_qshared::shared::qboolean;

use abi_transport::generic::InboundVmCall;

// Flow:
//
//   executable --vmMain(GAME_CONSOLE_COMMAND, ...)--> jampgame
//   jampgame   --ConsoleCommand()-------------------> process command from engine args
//   jampgame   --return qboolean--------------------> executable
//
// `GAME_CONSOLE_COMMAND` is an inbound executable-to-game call raised when the
// engine has a console command that was not handled as a builtin command.

/// `GAME_CONSOLE_COMMAND` MP game exports vmMain ABI token.
///
/// Raven: ( void );
/// Raven: ConsoleCommand will be called when a command has been issued
/// Raven: that is not recognized as a builtin function.
/// Raven: The game can issue trap_argc() / trap_argv() commands to get the command
/// Raven: and parameters.  Return qfalse if the game doesn't recognize it as a command.
/// Source: `oracle/oracle/codemp/game/g_public.h:758`
pub struct GameConsoleCommand;

impl InboundVmCall for GameConsoleCommand {
    type Command = MpGameExport;
    type Args = ();
    type Output = qboolean;

    const COMMAND: MpGameExport = MpGameExport::GAME_CONSOLE_COMMAND;
}
