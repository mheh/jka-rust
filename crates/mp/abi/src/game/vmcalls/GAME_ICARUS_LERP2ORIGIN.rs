use super::super::MpGameExport;

use abi_transport::generic::InboundVmCall;

/// `GAME_ICARUS_LERP2ORIGIN` MP game exports vmMain ABI token.
///
/// Source (enum): `oracle/oracle/codemp/game/g_public.h:774`
/// Source (args): `oracle/oracle/codemp/game/g_main.c:581`
/// Source (output): `oracle/oracle/codemp/game/g_main.c:584`
/// Source (call site): `oracle/oracle/codemp/icarus/Q3_Interface.cpp:806`
pub struct GameIcarusLerp2Origin;

impl InboundVmCall for GameIcarusLerp2Origin {
    type Command = MpGameExport;
    type Args = ();
    type Output = ();

    const COMMAND: MpGameExport = MpGameExport::GAME_ICARUS_LERP2ORIGIN;
}
