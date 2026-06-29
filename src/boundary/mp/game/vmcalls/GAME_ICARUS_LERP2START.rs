use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_ICARUS_LERP2START` MP game exports vmMain boundary token.
///
/// Source (enum): `oracle/oracle/codemp/game/g_public.h:777`
/// Source (args): `oracle/oracle/codemp/game/g_main.c:598`
/// Source (output): `oracle/oracle/codemp/game/g_main.c:601`
/// Source (call site): `oracle/oracle/codemp/icarus/Q3_Interface.cpp:846`
pub struct GameIcarusLerp2Start;

impl InboundVmCall for GameIcarusLerp2Start {
    type Command = GameExport;
    type Args = ();
    type Output = ();

    const COMMAND: GameExport = GameExport::GAME_ICARUS_LERP2START;
}
