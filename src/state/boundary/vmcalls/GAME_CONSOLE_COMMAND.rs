use crate::ffi::types::qboolean;
use crate::ffi::GameExport;

use super::super::generic::InboundVmCall;

// Flow:
//
//   executable --vmMain(GAME_CONSOLE_COMMAND, ...)--> jampgame
//   jampgame   --ConsoleCommand()-------------------> process command from engine args
//   jampgame   --return qboolean--------------------> executable
//
// `GAME_CONSOLE_COMMAND` is an inbound executable-to-game call raised when the
// engine has a console command that was not handled as a builtin command.

/// `GAME_CONSOLE_COMMAND` asks game code to handle the current console command.
pub struct GameConsoleCommand;

impl InboundVmCall for GameConsoleCommand {
    type Args = ();
    type Output = qboolean;

    const COMMAND: GameExport = GameExport::GAME_CONSOLE_COMMAND;
}
