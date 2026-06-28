use crate::ffi::GameExport;

use super::super::generic::{InboundVmCall, RawVmCallArgs};

/// `GAME_ICARUS_SET` inbound executable-to-game `vmMain` call.
pub struct GameIcarusSet;

impl InboundVmCall for GameIcarusSet {
    type Args = RawVmCallArgs;
    type Output = isize;

    const COMMAND: GameExport = GameExport::GAME_ICARUS_SET;
}
