use core::ffi::c_int;

use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

// Flow:
//
//   executable --vmMain(GAME_CLIENT_COMMAND, clientNum, ...)--> jampgame
//   jampgame   --ClientCommand(clientNum)-------------------> process client command
//   jampgame   --return 0-----------------------------------> executable
//
// `GAME_CLIENT_COMMAND` is an inbound executable-to-game call raised when the
// engine has a client command for game code to process.

/// Arguments for `GAME_CLIENT_COMMAND`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameClientCommandArgs {
    client_num: c_int,
}

impl GameClientCommandArgs {
    pub const fn new(client_num: c_int) -> Self {
        Self { client_num }
    }

    pub const fn client_num(self) -> c_int {
        self.client_num
    }
}

/// `GAME_CLIENT_COMMAND` processes a client-issued command.
pub struct GameClientCommand;

impl InboundVmCall for GameClientCommand {
    type Command = GameExport;
    type Args = GameClientCommandArgs;
    type Output = ();

    const COMMAND: GameExport = GameExport::GAME_CLIENT_COMMAND;
}
