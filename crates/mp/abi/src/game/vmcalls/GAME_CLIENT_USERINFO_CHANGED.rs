use core::ffi::c_int;

use super::super::MpGameExport;

use abi_transport::generic::{
    word_to_c_int, DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport,
};

// Flow:
//
//   executable --vmMain(GAME_CLIENT_USERINFO_CHANGED, clientNum, ...)--> jampgame
//   jampgame   --ClientUserinfoChanged(clientNum)-------------------> refresh client info
//   jampgame   --return 0-------------------------------------------> executable
//
// `GAME_CLIENT_USERINFO_CHANGED` is an inbound executable-to-game call raised
// when the engine tells game code a client's userinfo changed.

/// Arguments for `GAME_CLIENT_USERINFO_CHANGED`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameClientUserinfoChangedArgs {
    client_num: c_int,
}

impl GameClientUserinfoChangedArgs {
    pub const fn new(client_num: c_int) -> Self {
        Self { client_num }
    }

    pub const fn client_num(self) -> c_int {
        self.client_num
    }
}

/// `GAME_CLIENT_USERINFO_CHANGED` MP game exports vmMain ABI token.
///
/// Raven: ( int clientNum );
/// Source: `oracle/oracle/codemp/game/g_public.h:748`
pub struct GameClientUserinfoChanged;

impl InboundVmCall for GameClientUserinfoChanged {
    type Command = MpGameExport;
    type Args = GameClientUserinfoChangedArgs;
    type Output = ();

    const COMMAND: MpGameExport = MpGameExport::GAME_CLIENT_USERINFO_CHANGED;
}

impl DecodeVmMain for GameClientUserinfoChanged {
    fn decode_vm_main(t: VmMainTransport) -> Self::Args {
        // `ClientUserinfoChanged( arg0 )` — g_main.c:529.
        GameClientUserinfoChangedArgs::new(word_to_c_int(t.arg(0)))
    }
}

impl EncodeVmMainReturn for GameClientUserinfoChanged {
    fn encode_return(_output: Self::Output) -> isize {
        // `ClientUserinfoChanged(...); return 0;` — g_main.c:529-530.
        0
    }
}
