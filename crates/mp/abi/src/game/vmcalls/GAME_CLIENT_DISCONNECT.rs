use core::ffi::c_int;

use super::super::MpGameExport;

use abi_transport::generic::{
    word_to_c_int, DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport,
};

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

impl DecodeVmMain for GameClientDisconnect {
    fn decode_vm_main(t: VmMainTransport) -> Self::Args {
        // `ClientDisconnect( arg0 )` — g_main.c:532.
        GameClientDisconnectArgs::new(word_to_c_int(t.arg(0)))
    }
}

impl EncodeVmMainReturn for GameClientDisconnect {
    fn encode_return(_output: Self::Output) -> isize {
        // `ClientDisconnect(...); return 0;` — g_main.c:532-533.
        0
    }
}
