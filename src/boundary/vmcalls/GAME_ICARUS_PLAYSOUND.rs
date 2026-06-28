use crate::ffi::GameExport;

use super::super::generic::{InboundVmCall, RawVmCallArgs};

/// `GAME_ICARUS_PLAYSOUND` inbound executable-to-game `vmMain` call.
pub struct GameIcarusPlaysound;

impl InboundVmCall for GameIcarusPlaysound {
    type Args = RawVmCallArgs;
    type Output = isize;

    const COMMAND: GameExport = GameExport::GAME_ICARUS_PLAYSOUND;
}
