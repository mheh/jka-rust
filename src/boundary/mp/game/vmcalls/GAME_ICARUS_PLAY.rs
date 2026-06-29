use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_ICARUS_PLAY` MP game exports vmMain boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:782`
pub struct GameIcarusPlay;

impl InboundVmCall for GameIcarusPlay {
    type Command = GameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: GameExport = GameExport::GAME_ICARUS_PLAY;
}
