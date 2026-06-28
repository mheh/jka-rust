use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_INIT` inbound executable-to-game `vmMain` call.
pub struct GameInit;

impl InboundVmCall for GameInit {
    type Command = GameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: GameExport = GameExport::GAME_INIT;
}
