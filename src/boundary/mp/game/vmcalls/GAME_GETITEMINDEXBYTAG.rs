use crate::ffi::GameExport;

use crate::boundary::generic::{InboundVmCall, RawVmCallArgs};

/// `GAME_GETITEMINDEXBYTAG` inbound executable-to-game `vmMain` call.
pub struct GameGetitemindexbytag;

impl InboundVmCall for GameGetitemindexbytag {
    type Command = GameExport;
    type Args = RawVmCallArgs;
    type Output = isize;

    const COMMAND: GameExport = GameExport::GAME_GETITEMINDEXBYTAG;
}
