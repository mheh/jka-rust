use crate::ffi::GameExport;

use crate::boundary::generic::{InboundVmCall, RawVmCallArgs};

/// `GAME_ICARUS_PLAYSOUND` inbound executable-to-game `vmMain` call.
pub struct GameIcarusPlaysound;

impl InboundVmCall for GameIcarusPlaysound {
    type Command = GameExport;
    type Args = RawVmCallArgs;
    type Output = isize;

    const COMMAND: GameExport = GameExport::GAME_ICARUS_PLAYSOUND;
}
