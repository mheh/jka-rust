use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_ICARUS_GETFLOAT` MP game exports vmMain boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:783`
pub struct GameIcarusGetfloat;

impl InboundVmCall for GameIcarusGetfloat {
    type Command = GameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: GameExport = GameExport::GAME_ICARUS_GETFLOAT;
}
