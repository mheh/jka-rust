use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_ICARUS_GETTAG` MP game exports vmMain boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:776`
pub struct GameIcarusGettag;

impl InboundVmCall for GameIcarusGettag {
    type Command = GameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: GameExport = GameExport::GAME_ICARUS_GETTAG;
}
