use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_NAV_CLEARPATHTOPOINT` MP game exports vmMain boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:788`
pub struct GameNavClearpathtopoint;

impl InboundVmCall for GameNavClearpathtopoint {
    type Command = GameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: GameExport = GameExport::GAME_NAV_CLEARPATHTOPOINT;
}
