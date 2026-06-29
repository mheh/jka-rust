use core::ffi::c_int;

use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

// Flow:
//
//   executable --vmMain(GAME_CLIENT_THINK, clientNum, ...)--> jampgame
//   jampgame   --ClientThink(clientNum, null_mut())---------> advance client input
//   jampgame   --return 0----------------------------------> executable
//
// `GAME_CLIENT_THINK` is an inbound executable-to-game call raised when the
// engine asks game code to process a client's current user command.

/// Arguments for `GAME_CLIENT_THINK`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameClientThinkArgs {
    client_num: c_int,
}

impl GameClientThinkArgs {
    pub const fn new(client_num: c_int) -> Self {
        Self { client_num }
    }

    pub const fn client_num(self) -> c_int {
        self.client_num
    }
}

/// `GAME_CLIENT_THINK` MP game exports vmMain boundary token.
///
/// Raven: ( int clientNum );
/// Source: `oracle/oracle/codemp/game/g_public.h:754`
pub struct GameClientThink;

impl InboundVmCall for GameClientThink {
    type Command = GameExport;
    type Args = GameClientThinkArgs;
    type Output = ();

    const COMMAND: GameExport = GameExport::GAME_CLIENT_THINK;
}
