use super::super::MpGameExport;

use abi_transport::generic::InboundVmCall;

/// `GAME_ICARUS_LERP2ANGLES` MP game exports vmMain ABI token.
///
/// Source (enum): `oracle/oracle/codemp/game/g_public.h:775`
/// Source (args): `oracle/oracle/codemp/game/g_main.c:587`
/// Source (output): `oracle/oracle/codemp/game/g_main.c:590`
/// Source (call site): `oracle/oracle/codemp/icarus/Q3_Interface.cpp:819`
pub struct GameIcarusLerp2Angles;

impl InboundVmCall for GameIcarusLerp2Angles {
    type Command = MpGameExport;
    type Args = ();
    type Output = ();

    const COMMAND: MpGameExport = MpGameExport::GAME_ICARUS_LERP2ANGLES;
}
