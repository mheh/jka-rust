use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_ICARUS_USE` MP game exports vmMain boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:779`
pub struct GameIcarusUse;

impl InboundVmCall for GameIcarusUse {
    type Command = GameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: GameExport = GameExport::GAME_ICARUS_USE;
}
