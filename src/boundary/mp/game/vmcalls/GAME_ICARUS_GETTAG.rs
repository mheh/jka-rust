use crate::ffi::GameExport;

use crate::boundary::generic::{InboundVmCall, RawVmCallArgs};

/// `GAME_ICARUS_GETTAG` inbound executable-to-game `vmMain` call.
pub struct GameIcarusGettag;

impl InboundVmCall for GameIcarusGettag {
    type Command = GameExport;
    type Args = RawVmCallArgs;
    type Output = isize;

    const COMMAND: GameExport = GameExport::GAME_ICARUS_GETTAG;
}
