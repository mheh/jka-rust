use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_NAV_CLEARLOS` MP game exports vmMain boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:789`
pub struct GameNavClearlos;

impl InboundVmCall for GameNavClearlos {
    type Command = GameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: GameExport = GameExport::GAME_NAV_CLEARLOS;
}
