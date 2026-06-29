use core::ffi::{c_char, c_int};

use crate::ffi::types::qboolean;
use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

// Flow:
//
//   executable --vmMain(GAME_CLIENT_CONNECT, clientNum, firstTime, isBot)--> jampgame
//   jampgame   --ClientConnect(clientNum, firstTime, isBot)-------------> gate client
//   jampgame   --return optional denial string or NULL-----------------> executable
//
/// Arguments for `GAME_CLIENT_CONNECT`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameClientConnectArgs {
    client_num: c_int,
    first_time: qboolean,
    is_bot: qboolean,
}

impl GameClientConnectArgs {
    pub const fn new(client_num: c_int, first_time: qboolean, is_bot: qboolean) -> Self {
        Self {
            client_num,
            first_time,
            is_bot,
        }
    }

    pub const fn client_num(self) -> c_int {
        self.client_num
    }

    pub const fn first_time(self) -> qboolean {
        self.first_time
    }

    pub const fn is_bot(self) -> qboolean {
        self.is_bot
    }
}

/// `GAME_CLIENT_CONNECT` MP game exports vmMain boundary token.
///
/// Source (enum): `oracle/oracle/codemp/game/g_public.h:742`
/// Source (args): `oracle/oracle/codemp/game/g_main.c:523`
/// Source (output): `oracle/oracle/codemp/game/g_main.c:523`
/// Source (call site): `oracle/oracle/codemp/server/sv_client.cpp:520`
pub struct GameClientConnect;

impl InboundVmCall for GameClientConnect {
    type Command = GameExport;
    type Args = GameClientConnectArgs;
    type Output = *const c_char;

    const COMMAND: GameExport = GameExport::GAME_CLIENT_CONNECT;
}
