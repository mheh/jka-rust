use crate::ffi::GameExport;

use super::super::generic::{InboundVmCall, RawVmCallArgs};

/// `GAME_ICARUS_SOUNDINDEX` inbound executable-to-game `vmMain` call.
pub struct GameIcarusSoundindex;

impl InboundVmCall for GameIcarusSoundindex {
    type Args = RawVmCallArgs;
    type Output = isize;

    const COMMAND: GameExport = GameExport::GAME_ICARUS_SOUNDINDEX;
}
