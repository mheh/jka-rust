use crate::ffi::GameExport;

use crate::abi::generic::InboundVmCall;

/// `GAME_ICARUS_LERP2POS` MP game exports vmMain ABI token.
///
/// Source (enum): `oracle/oracle/codemp/game/g_public.h:773`
/// Source (args): `oracle/oracle/codemp/game/g_main.c:568`
/// Source (output): `oracle/oracle/codemp/game/g_main.c:576`
/// Source (call site): `oracle/oracle/codemp/icarus/Q3_Interface.cpp:786`
pub struct GameIcarusLerp2Pos;

impl InboundVmCall for GameIcarusLerp2Pos {
    type Command = GameExport;
    type Args = ();
    type Output = ();

    const COMMAND: GameExport = GameExport::GAME_ICARUS_LERP2POS;
}
