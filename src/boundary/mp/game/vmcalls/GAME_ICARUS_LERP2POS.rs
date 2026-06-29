use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_ICARUS_LERP2POS` MP game exports vmMain boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:773`
pub struct GameIcarusLerp2Pos;

impl InboundVmCall for GameIcarusLerp2Pos {
    type Command = GameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: GameExport = GameExport::GAME_ICARUS_LERP2POS;
}
