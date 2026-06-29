use core::ffi::c_int;

use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_ICARUS_GETSTRING` MP game exports vmMain boundary token.
///
/// Source (enum): `oracle/oracle/codemp/game/g_public.h:785`
/// Source (args): `oracle/oracle/codemp/game/g_main.c:644`
/// Source (output): `oracle/oracle/codemp/game/g_main.c:647`
/// Source (call site): `oracle/oracle/codemp/icarus/Q3_Interface.cpp:941`
pub struct GameIcarusGetstring;

impl InboundVmCall for GameIcarusGetstring {
    type Command = GameExport;
    type Args = ();
    type Output = c_int;

    const COMMAND: GameExport = GameExport::GAME_ICARUS_GETSTRING;
}
