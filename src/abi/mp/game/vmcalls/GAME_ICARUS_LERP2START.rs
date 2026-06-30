use super::super::MpGameExport;

use crate::abi::generic::InboundVmCall;

/// `GAME_ICARUS_LERP2START` MP game exports vmMain ABI token.
///
/// Source (enum): `oracle/oracle/codemp/game/g_public.h:777`
/// Source (args): `oracle/oracle/codemp/game/g_main.c:598`
/// Source (output): `oracle/oracle/codemp/game/g_main.c:601`
/// Source (call site): `oracle/oracle/codemp/icarus/Q3_Interface.cpp:846`
pub struct GameIcarusLerp2Start;

impl InboundVmCall for GameIcarusLerp2Start {
    type Command = MpGameExport;
    type Args = ();
    type Output = ();

    const COMMAND: MpGameExport = MpGameExport::GAME_ICARUS_LERP2START;
}
