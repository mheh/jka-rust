use crate::ffi::GameExport;

use crate::boundary::generic::{InboundVmCall, RawVmCallArgs};

/// `GAME_ICARUS_PLAY` inbound executable-to-game `vmMain` call.
pub struct GameIcarusPlay;

impl InboundVmCall for GameIcarusPlay {
    type Command = GameExport;
    type Args = RawVmCallArgs;
    type Output = isize;

    const COMMAND: GameExport = GameExport::GAME_ICARUS_PLAY;
}
