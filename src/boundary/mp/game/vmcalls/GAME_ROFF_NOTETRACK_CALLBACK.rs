use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_ROFF_NOTETRACK_CALLBACK` inbound executable-to-game `vmMain` call.
pub struct GameRoffNotetrackCallback;

impl InboundVmCall for GameRoffNotetrackCallback {
    type Command = GameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: GameExport = GameExport::GAME_ROFF_NOTETRACK_CALLBACK;
}
