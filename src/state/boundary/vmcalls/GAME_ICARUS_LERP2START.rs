use crate::ffi::GameExport;

use super::super::generic::{InboundVmCall, RawVmCallArgs};

/// `GAME_ICARUS_LERP2START` inbound executable-to-game `vmMain` call.
pub struct GameIcarusLerp2Start;

impl InboundVmCall for GameIcarusLerp2Start {
    type Args = RawVmCallArgs;
    type Output = isize;

    const COMMAND: GameExport = GameExport::GAME_ICARUS_LERP2START;
}
