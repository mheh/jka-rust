use super::super::MpGameExport;

use abi_transport::generic::InboundVmCall;

/// `GAME_ICARUS_USE` MP game exports vmMain ABI token.
///
/// Source (enum): `oracle/oracle/codemp/game/g_public.h:779`
/// Source (args): `oracle/oracle/codemp/game/g_main.c:610`
/// Source (output): `oracle/oracle/codemp/game/g_main.c:613`
/// Source (call site): `oracle/oracle/codemp/icarus/Q3_Interface.cpp:867`
pub struct GameIcarusUse;

impl InboundVmCall for GameIcarusUse {
    type Command = MpGameExport;
    type Args = ();
    type Output = ();

    const COMMAND: MpGameExport = MpGameExport::GAME_ICARUS_USE;
}
