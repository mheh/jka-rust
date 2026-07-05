use super::super::MpGameExport;

use abi_transport::generic::{DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport};

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

impl DecodeVmMain for GameSpawnRmgEntity {
    // `GAME_SPAWN_RMG_ENTITY` takes no vmMain arg words — g_main.c:550-554.
    fn decode_vm_main(_t: VmMainTransport) -> Self::Args {}
}

impl EncodeVmMainReturn for GameSpawnRmgEntity {
    fn encode_return(_output: Self::Output) -> isize {
        // `... return 0;` — g_main.c:555.
        0
    }
}
