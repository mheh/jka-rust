use crate::ffi::GameExport;

use super::super::generic::{InboundVmCall, RawVmCallArgs};

/// `GAME_ICARUS_LERP2ANGLES` inbound executable-to-game `vmMain` call.
pub struct GameIcarusLerp2Angles;

impl InboundVmCall for GameIcarusLerp2Angles {
    type Args = RawVmCallArgs;
    type Output = isize;

    const COMMAND: GameExport = GameExport::GAME_ICARUS_LERP2ANGLES;
}
