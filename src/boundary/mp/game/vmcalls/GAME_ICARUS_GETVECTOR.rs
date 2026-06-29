use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_ICARUS_GETVECTOR` MP game exports vmMain boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:784`
pub struct GameIcarusGetvector;

impl InboundVmCall for GameIcarusGetvector {
    type Command = GameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: GameExport = GameExport::GAME_ICARUS_GETVECTOR;
}
