use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_ICARUS_GETSETIDFORSTRING` MP game exports vmMain boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:787`
pub struct GameIcarusGetsetidforstring;

impl InboundVmCall for GameIcarusGetsetidforstring {
    type Command = GameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: GameExport = GameExport::GAME_ICARUS_GETSETIDFORSTRING;
}
