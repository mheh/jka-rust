use crate::ffi::types::qboolean;
use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_ICARUS_SET` MP game exports vmMain boundary token.
///
/// Source (enum): `oracle/oracle/codemp/game/g_public.h:772`
/// Source (args): `oracle/oracle/codemp/game/g_main.c:563`
/// Source (output): `oracle/oracle/codemp/game/g_main.c:566`
/// Source (call site): `oracle/oracle/codemp/icarus/Q3_Interface.cpp:397`
pub struct GameIcarusSet;

impl InboundVmCall for GameIcarusSet {
    type Command = GameExport;
    type Args = ();
    type Output = qboolean;

    const COMMAND: GameExport = GameExport::GAME_ICARUS_SET;
}
