use super::super::MpGameExport;

use abi_transport::generic::InboundVmCall;

/// `GAME_NAV_FINDCOMBATPOINTWAYPOINTS` MP game exports vmMain ABI token.
///
/// Source (enum): `oracle/oracle/codemp/game/g_public.h:796`
/// Source (args): `oracle/oracle/codemp/game/g_main.c:688`
/// Source (output): `oracle/oracle/codemp/game/g_main.c:689`
/// Source (call site): `oracle/oracle/codemp/server/NPCNav/gameCallbacks.cpp:48`
pub struct GameNavFindcombatpointwaypoints;

impl InboundVmCall for GameNavFindcombatpointwaypoints {
    type Command = MpGameExport;
    type Args = ();
    type Output = ();

    const COMMAND: MpGameExport = MpGameExport::GAME_NAV_FINDCOMBATPOINTWAYPOINTS;
}
