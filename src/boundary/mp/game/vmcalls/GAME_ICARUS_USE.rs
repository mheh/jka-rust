use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_ICARUS_USE` MP game exports vmMain boundary token.
///
/// Source (enum): `oracle/oracle/codemp/game/g_public.h:779`
/// Source (args): `oracle/oracle/codemp/game/g_main.c:610`
/// Source (output): `oracle/oracle/codemp/game/g_main.c:613`
/// Source (call site): `oracle/oracle/codemp/icarus/Q3_Interface.cpp:867`
pub struct GameIcarusUse;

impl InboundVmCall for GameIcarusUse {
    type Command = GameExport;
    type Args = ();
    type Output = ();

    const COMMAND: GameExport = GameExport::GAME_ICARUS_USE;
}
