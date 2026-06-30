use super::super::MpGameExport;
use crate::shared::qboolean;

use crate::abi::generic::InboundVmCall;

/// `GAME_ICARUS_SET` MP game exports vmMain ABI token.
///
/// Source (enum): `oracle/oracle/codemp/game/g_public.h:772`
/// Source (args): `oracle/oracle/codemp/game/g_main.c:563`
/// Source (output): `oracle/oracle/codemp/game/g_main.c:566`
/// Source (call site): `oracle/oracle/codemp/icarus/Q3_Interface.cpp:397`
pub struct GameIcarusSet;

impl InboundVmCall for GameIcarusSet {
    type Command = MpGameExport;
    type Args = ();
    type Output = qboolean;

    const COMMAND: MpGameExport = MpGameExport::GAME_ICARUS_SET;
}
