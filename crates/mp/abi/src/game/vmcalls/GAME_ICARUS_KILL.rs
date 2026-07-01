use super::super::MpGameExport;

use abi_transport::generic::InboundVmCall;

/// `GAME_ICARUS_KILL` MP game exports vmMain ABI token.
///
/// Source (enum): `oracle/oracle/codemp/game/g_public.h:780`
/// Source (args): `oracle/oracle/codemp/game/g_main.c:616`
/// Source (output): `oracle/oracle/codemp/game/g_main.c:619`
/// Source (call site): `oracle/oracle/codemp/icarus/Q3_Interface.cpp:877`
pub struct GameIcarusKill;

impl InboundVmCall for GameIcarusKill {
    type Command = MpGameExport;
    type Args = ();
    type Output = ();

    const COMMAND: MpGameExport = MpGameExport::GAME_ICARUS_KILL;
}
