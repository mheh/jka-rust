use crate::ffi::GameExport;

use super::super::generic::{InboundVmCall, RawVmCallArgs};

/// `GAME_ROFF_NOTETRACK_CALLBACK` inbound executable-to-game `vmMain` call.
pub struct GameRoffNotetrackCallback;

impl InboundVmCall for GameRoffNotetrackCallback {
    type Args = RawVmCallArgs;
    type Output = isize;

    const COMMAND: GameExport = GameExport::GAME_ROFF_NOTETRACK_CALLBACK;
}
