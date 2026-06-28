use crate::ffi::GameExport;

use crate::boundary::generic::{InboundVmCall, RawVmCallArgs};

/// `GAME_ICARUS_REMOVE` inbound executable-to-game `vmMain` call.
pub struct GameIcarusRemove;

impl InboundVmCall for GameIcarusRemove {
    type Command = GameExport;
    type Args = RawVmCallArgs;
    type Output = isize;

    const COMMAND: GameExport = GameExport::GAME_ICARUS_REMOVE;
}
