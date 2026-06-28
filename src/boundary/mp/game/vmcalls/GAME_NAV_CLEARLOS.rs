use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_NAV_CLEARLOS` inbound executable-to-game `vmMain` call.
pub struct GameNavClearlos;

impl InboundVmCall for GameNavClearlos {
    type Command = GameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: GameExport = GameExport::GAME_NAV_CLEARLOS;
}
