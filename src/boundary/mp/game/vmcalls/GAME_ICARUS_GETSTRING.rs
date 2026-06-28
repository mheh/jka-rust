use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_ICARUS_GETSTRING` inbound executable-to-game `vmMain` call.
pub struct GameIcarusGetstring;

impl InboundVmCall for GameIcarusGetstring {
    type Command = GameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: GameExport = GameExport::GAME_ICARUS_GETSTRING;
}
