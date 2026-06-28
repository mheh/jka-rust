use crate::ffi::GameExport;

use super::super::generic::{InboundVmCall, RawVmCallArgs};

/// `GAME_ICARUS_KILL` inbound executable-to-game `vmMain` call.
pub struct GameIcarusKill;

impl InboundVmCall for GameIcarusKill {
    type Args = RawVmCallArgs;
    type Output = isize;

    const COMMAND: GameExport = GameExport::GAME_ICARUS_KILL;
}
