use crate::ffi::GameExport;

use super::super::generic::{InboundVmCall, RawVmCallArgs};

/// `GAME_ICARUS_USE` inbound executable-to-game `vmMain` call.
pub struct GameIcarusUse;

impl InboundVmCall for GameIcarusUse {
    type Args = RawVmCallArgs;
    type Output = isize;

    const COMMAND: GameExport = GameExport::GAME_ICARUS_USE;
}
