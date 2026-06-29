use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_ICARUS_REMOVE` MP game exports vmMain boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:781`
pub struct GameIcarusRemove;

impl InboundVmCall for GameIcarusRemove {
    type Command = GameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: GameExport = GameExport::GAME_ICARUS_REMOVE;
}
