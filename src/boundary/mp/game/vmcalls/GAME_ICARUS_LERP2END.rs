use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_ICARUS_LERP2END` MP game exports vmMain boundary token.
///
/// Source (enum): `oracle/oracle/codemp/game/g_public.h:778`
/// Source (args): `oracle/oracle/codemp/game/g_main.c:604`
/// Source (output): `oracle/oracle/codemp/game/g_main.c:607`
/// Source (call site): `oracle/oracle/codemp/icarus/Q3_Interface.cpp:857`
pub struct GameIcarusLerp2End;

impl InboundVmCall for GameIcarusLerp2End {
    type Command = GameExport;
    type Args = ();
    type Output = ();

    const COMMAND: GameExport = GameExport::GAME_ICARUS_LERP2END;
}
