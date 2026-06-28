use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_ICARUS_SOUNDINDEX` inbound executable-to-game `vmMain` call.
pub struct GameIcarusSoundindex;

impl InboundVmCall for GameIcarusSoundindex {
    type Command = GameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: GameExport = GameExport::GAME_ICARUS_SOUNDINDEX;
}
