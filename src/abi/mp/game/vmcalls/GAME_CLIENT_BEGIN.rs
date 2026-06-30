use core::ffi::c_int;

use super::super::MpGameExport;

use crate::abi::generic::InboundVmCall;

// Flow:
//
//   executable --vmMain(GAME_CLIENT_BEGIN, clientNum, ...)--> jampgame
//   jampgame   --ClientBegin(clientNum, QTRUE)-------------> begin client session
//   jampgame   --return 0----------------------------------> executable
//
// `GAME_CLIENT_BEGIN` is an inbound executable-to-game call raised when the
// engine asks game code to finish placing a client into the level.

/// Arguments for `GAME_CLIENT_BEGIN`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameClientBeginArgs {
    client_num: c_int,
}

impl GameClientBeginArgs {
    pub const fn new(client_num: c_int) -> Self {
        Self { client_num }
    }

    pub const fn client_num(self) -> c_int {
        self.client_num
    }
}

/// `GAME_CLIENT_BEGIN` MP game exports vmMain ABI token.
///
/// Raven: ( int clientNum );
/// Source: `oracle/oracle/codemp/game/g_public.h:746`
pub struct GameClientBegin;

impl InboundVmCall for GameClientBegin {
    type Command = MpGameExport;
    type Args = GameClientBeginArgs;
    type Output = ();

    const COMMAND: MpGameExport = MpGameExport::GAME_CLIENT_BEGIN;
}
