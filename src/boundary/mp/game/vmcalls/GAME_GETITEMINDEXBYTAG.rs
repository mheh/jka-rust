use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_GETITEMINDEXBYTAG` MP game exports vmMain boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:798`
pub struct GameGetitemindexbytag;

impl InboundVmCall for GameGetitemindexbytag {
    type Command = GameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: GameExport = GameExport::GAME_GETITEMINDEXBYTAG;
}
