use crate::ffi::GameExport;

use crate::abi::generic::InboundVmCall;

/// `GAME_ICARUS_REMOVE` MP game exports vmMain ABI token.
///
/// Source (enum): `oracle/oracle/codemp/game/g_public.h:781`
/// Source (args): `oracle/oracle/codemp/game/g_main.c:622`
/// Source (output): `oracle/oracle/codemp/game/g_main.c:625`
/// Source (call site): `oracle/oracle/codemp/icarus/Q3_Interface.cpp:887`
pub struct GameIcarusRemove;

impl InboundVmCall for GameIcarusRemove {
    type Command = GameExport;
    type Args = ();
    type Output = ();

    const COMMAND: GameExport = GameExport::GAME_ICARUS_REMOVE;
}
