use super::super::MpGameExport;

use abi_transport::generic::InboundVmCall;

/// `GAME_ICARUS_PLAY` MP game exports vmMain ABI token.
///
/// Source (enum): `oracle/oracle/codemp/game/g_public.h:782`
/// Source (args): `oracle/oracle/codemp/game/g_main.c:628`
/// Source (output): `oracle/oracle/codemp/game/g_main.c:631`
/// Source (call site): `oracle/oracle/codemp/icarus/Q3_Interface.cpp:899`
pub struct GameIcarusPlay;

impl InboundVmCall for GameIcarusPlay {
    type Command = MpGameExport;
    type Args = ();
    type Output = ();

    const COMMAND: MpGameExport = MpGameExport::GAME_ICARUS_PLAY;
}
