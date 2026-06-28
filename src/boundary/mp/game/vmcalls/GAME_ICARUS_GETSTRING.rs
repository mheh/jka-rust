use crate::ffi::GameExport;

use crate::boundary::generic::{InboundVmCall, RawVmCallArgs};

/// `GAME_ICARUS_GETSTRING` inbound executable-to-game `vmMain` call.
pub struct GameIcarusGetstring;

impl InboundVmCall for GameIcarusGetstring {
    type Command = GameExport;
    type Args = RawVmCallArgs;
    type Output = isize;

    const COMMAND: GameExport = GameExport::GAME_ICARUS_GETSTRING;
}
