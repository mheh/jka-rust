use super::super::MpGameExport;

use abi_transport::generic::InboundVmCall;

/// `GAME_SPAWN_RMG_ENTITY` MP game exports vmMain ABI token.
///
/// Raven: rwwRMG - added
/// Raven: rww - icarus callbacks
/// Source (enum): `oracle/oracle/codemp/game/g_public.h:768`
/// Source (args): `oracle/oracle/codemp/game/g_main.c:550`
/// Source (output): `oracle/oracle/codemp/game/g_main.c:553`
/// Source (call site): `oracle/oracle/RMG/RM_Instance_BSP.cpp:264`
pub struct GameSpawnRmgEntity;

impl InboundVmCall for GameSpawnRmgEntity {
    type Command = MpGameExport;
    type Args = ();
    type Output = ();

    const COMMAND: MpGameExport = MpGameExport::GAME_SPAWN_RMG_ENTITY;
}
