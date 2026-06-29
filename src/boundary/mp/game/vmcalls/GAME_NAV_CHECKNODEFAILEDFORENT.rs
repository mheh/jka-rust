use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_NAV_CHECKNODEFAILEDFORENT` MP game exports vmMain boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:791`
pub struct GameNavChecknodefailedforent;

impl InboundVmCall for GameNavChecknodefailedforent {
    type Command = GameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: GameExport = GameExport::GAME_NAV_CHECKNODEFAILEDFORENT;
}
