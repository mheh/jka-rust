use super::super::MpGameExport;

use abi_transport::generic::{DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport};

/// `GAME_NAV_FINDCOMBATPOINTWAYPOINTS` MP game exports vmMain ABI token.
///
/// Source (enum): `oracle/codemp/game/g_public.h:796`
/// Source (args): `oracle/codemp/game/g_main.c:688`
/// Source (output): `oracle/codemp/game/g_main.c:689`
/// Source (call site): `oracle/codemp/server/NPCNav/gameCallbacks.cpp:48`
pub struct GameNavFindcombatpointwaypoints;

impl InboundVmCall for GameNavFindcombatpointwaypoints {
    type Command = MpGameExport;
    type Args = ();
    type Output = ();

    const COMMAND: MpGameExport = MpGameExport::GAME_NAV_FINDCOMBATPOINTWAYPOINTS;
}

impl DecodeVmMain for GameNavFindcombatpointwaypoints {
    // Takes no vmMain arg words — g_main.c:688-689.
    fn decode_vm_main(_t: VmMainTransport) -> Self::Args {}
}

impl EncodeVmMainReturn for GameNavFindcombatpointwaypoints {
    fn encode_return(_output: Self::Output) -> isize {
        // `CP_FindCombatPointWaypoints(); return 0;` — g_main.c:688-689.
        0
    }
}
