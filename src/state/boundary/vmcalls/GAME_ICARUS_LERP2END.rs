use crate::ffi::GameExport;

use super::super::generic::{InboundVmCall, RawVmCallArgs};

/// `GAME_ICARUS_LERP2END` inbound executable-to-game `vmMain` call.
pub struct GameIcarusLerp2End;

impl InboundVmCall for GameIcarusLerp2End {
    type Args = RawVmCallArgs;
    type Output = isize;

    const COMMAND: GameExport = GameExport::GAME_ICARUS_LERP2END;
}
