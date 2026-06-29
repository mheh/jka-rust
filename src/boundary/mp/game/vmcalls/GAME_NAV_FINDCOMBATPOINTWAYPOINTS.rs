use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_NAV_FINDCOMBATPOINTWAYPOINTS` MP game exports vmMain boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:796`
pub struct GameNavFindcombatpointwaypoints;

impl InboundVmCall for GameNavFindcombatpointwaypoints {
    type Command = GameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: GameExport = GameExport::GAME_NAV_FINDCOMBATPOINTWAYPOINTS;
}
