use core::ffi::{c_char, c_int};

use super::super::MpGameExport;
use crate::shared::qboolean;

use crate::abi::generic::InboundVmCall;

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

/// `GAME_CLIENT_CONNECT` MP game exports vmMain ABI token.
///
/// Source (enum): `oracle/oracle/codemp/game/g_public.h:742`
/// Source (args): `oracle/oracle/codemp/game/g_main.c:523`
/// Source (output): `oracle/oracle/codemp/game/g_main.c:523`
/// Source (call site): `oracle/oracle/codemp/server/sv_client.cpp:520`
pub struct GameClientConnect;

impl InboundVmCall for GameClientConnect {
    type Command = MpGameExport;
    type Args = GameClientConnectArgs;
    type Output = *const c_char;

    const COMMAND: MpGameExport = MpGameExport::GAME_CLIENT_CONNECT;
}
