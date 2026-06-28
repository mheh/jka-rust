use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_ICARUS_PLAYSOUND` inbound executable-to-game `vmMain` call.
pub struct GameIcarusPlaysound;

impl InboundVmCall for GameIcarusPlaysound {
    type Command = GameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: GameExport = GameExport::GAME_ICARUS_PLAYSOUND;
}
