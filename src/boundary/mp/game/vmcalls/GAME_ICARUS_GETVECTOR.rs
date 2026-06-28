use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_ICARUS_GETVECTOR` inbound executable-to-game `vmMain` call.
pub struct GameIcarusGetvector;

impl InboundVmCall for GameIcarusGetvector {
    type Command = GameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: GameExport = GameExport::GAME_ICARUS_GETVECTOR;
}
