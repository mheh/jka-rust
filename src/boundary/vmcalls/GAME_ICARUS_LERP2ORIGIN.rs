use crate::ffi::GameExport;

use super::super::generic::{InboundVmCall, RawVmCallArgs};

/// `GAME_ICARUS_LERP2ORIGIN` inbound executable-to-game `vmMain` call.
pub struct GameIcarusLerp2Origin;

impl InboundVmCall for GameIcarusLerp2Origin {
    type Args = RawVmCallArgs;
    type Output = isize;

    const COMMAND: GameExport = GameExport::GAME_ICARUS_LERP2ORIGIN;
}
