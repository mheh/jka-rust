use core::ffi::c_int;

use super::super::MpGameExport;

use crate::abi::generic::InboundVmCall;

// Flow:
//
//   executable --vmMain(GAME_CLIENT_DISCONNECT, clientNum, ...)--> jampgame
//   jampgame   --ClientDisconnect(clientNum)-------------------> remove client state
//   jampgame   --return 0--------------------------------------> executable
//
// `GAME_CLIENT_DISCONNECT` is an inbound executable-to-game call raised when
// the engine tells game code that a client is leaving.

/// Arguments for `GAME_CLIENT_DISCONNECT`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameClientDisconnectArgs {
    client_num: c_int,
}

impl GameClientDisconnectArgs {
    pub const fn new(client_num: c_int) -> Self {
        Self { client_num }
    }

    pub const fn client_num(self) -> c_int {
        self.client_num
    }
}

/// `GAME_CLIENT_DISCONNECT` MP game exports vmMain ABI token.
///
/// Raven: ( int clientNum );
/// Source: `oracle/oracle/codemp/game/g_public.h:750`
pub struct GameClientDisconnect;

impl InboundVmCall for GameClientDisconnect {
    type Command = MpGameExport;
    type Args = GameClientDisconnectArgs;
    type Output = ();

    const COMMAND: MpGameExport = MpGameExport::GAME_CLIENT_DISCONNECT;
}
