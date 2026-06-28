use crate::ffi::GameExport;

use crate::boundary::generic::{InboundVmCall, RawVmCallArgs};

/// `GAME_ICARUS_GETFLOAT` inbound executable-to-game `vmMain` call.
pub struct GameIcarusGetfloat;

impl InboundVmCall for GameIcarusGetfloat {
    type Command = GameExport;
    type Args = RawVmCallArgs;
    type Output = isize;

    const COMMAND: GameExport = GameExport::GAME_ICARUS_GETFLOAT;
}
