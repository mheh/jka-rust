use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_ICARUS_LERP2ORIGIN` MP game exports vmMain boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:774`
pub struct GameIcarusLerp2Origin;

impl InboundVmCall for GameIcarusLerp2Origin {
    type Command = GameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: GameExport = GameExport::GAME_ICARUS_LERP2ORIGIN;
}
