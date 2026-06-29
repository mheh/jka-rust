use core::ffi::c_int;

use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

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

/// `GAME_CLIENT_USERINFO_CHANGED` MP game exports vmMain boundary token.
///
/// Raven: ( int clientNum );
/// Source: `oracle/oracle/codemp/game/g_public.h:748`
pub struct GameClientUserinfoChanged;

impl InboundVmCall for GameClientUserinfoChanged {
    type Command = GameExport;
    type Args = GameClientUserinfoChangedArgs;
    type Output = ();

    const COMMAND: GameExport = GameExport::GAME_CLIENT_USERINFO_CHANGED;
}
