use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_ICARUS_USE` inbound executable-to-game `vmMain` call.
pub struct GameIcarusUse;

impl InboundVmCall for GameIcarusUse {
    type Command = GameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: GameExport = GameExport::GAME_ICARUS_USE;
}
