use core::ffi::c_int;

use super::super::MpGameExport;

use crate::abi::generic::InboundVmCall;

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

/// `GAME_CLIENT_COMMAND` MP game exports vmMain ABI token.
///
/// Raven: ( int clientNum );
/// Source: `oracle/oracle/codemp/game/g_public.h:752`
pub struct GameClientCommand;

impl InboundVmCall for GameClientCommand {
    type Command = MpGameExport;
    type Args = GameClientCommandArgs;
    type Output = ();

    const COMMAND: MpGameExport = MpGameExport::GAME_CLIENT_COMMAND;
}
