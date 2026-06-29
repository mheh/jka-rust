use core::ffi::c_int;

use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_ICARUS_GETTAG` MP game exports vmMain boundary token.
///
/// Source (enum): `oracle/oracle/codemp/game/g_public.h:776`
/// Source (args): `oracle/oracle/codemp/game/g_main.c:593`
/// Source (output): `oracle/oracle/codemp/game/g_main.c:596`
/// Source (call site): `oracle/oracle/codemp/icarus/Q3_Interface.cpp:833`
pub struct GameIcarusGettag;

impl InboundVmCall for GameIcarusGettag {
    type Command = GameExport;
    type Args = ();
    type Output = c_int;

    const COMMAND: GameExport = GameExport::GAME_ICARUS_GETTAG;
}
