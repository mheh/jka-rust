use crate::ffi::GameExport;

use crate::abi::generic::InboundVmCall;

/// `GAME_ICARUS_SOUNDINDEX` MP game exports vmMain ABI token.
///
/// Source (enum): `oracle/oracle/codemp/game/g_public.h:786`
/// Source (args): `oracle/oracle/codemp/game/g_main.c:659`
/// Source (output): `oracle/oracle/codemp/game/g_main.c:662`
/// Source (call site): `oracle/oracle/codemp/icarus/GameInterface.cpp:406`
pub struct GameIcarusSoundindex;

impl InboundVmCall for GameIcarusSoundindex {
    type Command = GameExport;
    type Args = ();
    type Output = ();

    const COMMAND: GameExport = GameExport::GAME_ICARUS_SOUNDINDEX;
}
