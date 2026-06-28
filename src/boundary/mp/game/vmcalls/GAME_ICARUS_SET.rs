use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_ICARUS_SET` inbound executable-to-game `vmMain` call.
pub struct GameIcarusSet;

impl InboundVmCall for GameIcarusSet {
    type Command = GameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: GameExport = GameExport::GAME_ICARUS_SET;
}
