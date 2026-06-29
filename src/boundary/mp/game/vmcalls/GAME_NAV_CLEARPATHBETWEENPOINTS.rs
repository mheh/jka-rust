use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_NAV_CLEARPATHBETWEENPOINTS` MP game exports vmMain boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:790`
pub struct GameNavClearpathbetweenpoints;

impl InboundVmCall for GameNavClearpathbetweenpoints {
    type Command = GameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: GameExport = GameExport::GAME_NAV_CLEARPATHBETWEENPOINTS;
}
