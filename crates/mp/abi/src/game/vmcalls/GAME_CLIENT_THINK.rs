use core::ffi::c_int;

use super::super::MpGameExport;

use abi_transport::generic::InboundVmCall;

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

/// `GAME_CLIENT_THINK` MP game exports vmMain ABI token.
///
/// Raven: ( int clientNum );
/// Source: `oracle/oracle/codemp/game/g_public.h:754`
pub struct GameClientThink;

impl InboundVmCall for GameClientThink {
    type Command = MpGameExport;
    type Args = GameClientThinkArgs;
    type Output = ();

    const COMMAND: MpGameExport = MpGameExport::GAME_CLIENT_THINK;
}
