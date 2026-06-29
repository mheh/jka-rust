use core::ffi::c_int;

use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_ICARUS_PLAYSOUND` MP game exports vmMain boundary token.
///
/// Source (enum): `oracle/oracle/codemp/game/g_public.h:771`
/// Source (args): `oracle/oracle/codemp/game/g_main.c:558`
/// Source (output): `oracle/oracle/codemp/game/g_main.c:560`
/// Source (call site): `oracle/oracle/codemp/icarus/Q3_Interface.cpp:322`
pub struct GameIcarusPlaysound;

impl InboundVmCall for GameIcarusPlaysound {
    type Command = GameExport;
    type Args = ();
    type Output = c_int;

    const COMMAND: GameExport = GameExport::GAME_ICARUS_PLAYSOUND;
}
