use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_ICARUS_GETTAG` inbound executable-to-game `vmMain` call.
pub struct GameIcarusGettag;

impl InboundVmCall for GameIcarusGettag {
    type Command = GameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: GameExport = GameExport::GAME_ICARUS_GETTAG;
}
