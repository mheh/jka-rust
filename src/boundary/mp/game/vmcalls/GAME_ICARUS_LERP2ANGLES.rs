use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_ICARUS_LERP2ANGLES` MP game exports vmMain boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:775`
pub struct GameIcarusLerp2Angles;

impl InboundVmCall for GameIcarusLerp2Angles {
    type Command = GameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: GameExport = GameExport::GAME_ICARUS_LERP2ANGLES;
}
