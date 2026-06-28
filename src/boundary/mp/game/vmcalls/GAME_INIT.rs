use crate::ffi::GameExport;

use crate::boundary::generic::{InboundVmCall, RawVmCallArgs};

/// `GAME_INIT` inbound executable-to-game `vmMain` call.
pub struct GameInit;

impl InboundVmCall for GameInit {
    type Command = GameExport;
    type Args = RawVmCallArgs;
    type Output = isize;

    const COMMAND: GameExport = GameExport::GAME_INIT;
}
