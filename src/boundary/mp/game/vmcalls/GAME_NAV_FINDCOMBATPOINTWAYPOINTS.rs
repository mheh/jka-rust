use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_NAV_FINDCOMBATPOINTWAYPOINTS` MP game exports vmMain boundary token.
///
/// Source (enum): `oracle/oracle/codemp/game/g_public.h:796`
/// Source (args): `oracle/oracle/codemp/game/g_main.c:688`
/// Source (output): `oracle/oracle/codemp/game/g_main.c:689`
/// Source (call site): `oracle/oracle/codemp/server/NPCNav/gameCallbacks.cpp:48`
pub struct GameNavFindcombatpointwaypoints;

impl InboundVmCall for GameNavFindcombatpointwaypoints {
    type Command = GameExport;
    type Args = ();
    type Output = ();

    const COMMAND: GameExport = GameExport::GAME_NAV_FINDCOMBATPOINTWAYPOINTS;
}
