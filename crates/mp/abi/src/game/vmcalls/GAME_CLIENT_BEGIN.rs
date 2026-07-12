use core::ffi::c_int;

use super::super::MpGameExport;

use abi_transport::generic::{
    word_to_c_int, DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport,
};

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
/// Source: `oracle/codemp/game/g_public.h:746`
pub struct GameClientBegin;

impl InboundVmCall for GameClientBegin {
    type Command = MpGameExport;
    type Args = GameClientBeginArgs;
    type Output = ();

    const COMMAND: MpGameExport = MpGameExport::GAME_CLIENT_BEGIN;
}

impl DecodeVmMain for GameClientBegin {
    fn decode_vm_main(t: VmMainTransport) -> Self::Args {
        // `ClientBegin( arg0, qtrue )` — g_main.c:535. The `qtrue`
        // `allowTeamReset` is supplied at the dispatch call site.
        GameClientBeginArgs::new(word_to_c_int(t.arg(0)))
    }
}

impl EncodeVmMainReturn for GameClientBegin {
    fn encode_return(_output: Self::Output) -> isize {
        // `ClientBegin(...); return 0;` — g_main.c:535-536.
        0
    }
}
