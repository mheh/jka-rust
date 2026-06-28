use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_GETITEMINDEXBYTAG` inbound executable-to-game `vmMain` call.
pub struct GameGetitemindexbytag;

impl InboundVmCall for GameGetitemindexbytag {
    type Command = GameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: GameExport = GameExport::GAME_GETITEMINDEXBYTAG;
}
