use crate::ffi::GameExport;

use super::super::generic::{InboundVmCall, RawVmCallArgs};

/// `GAME_ICARUS_LERP2POS` inbound executable-to-game `vmMain` call.
pub struct GameIcarusLerp2Pos;

impl InboundVmCall for GameIcarusLerp2Pos {
    type Args = RawVmCallArgs;
    type Output = isize;

    const COMMAND: GameExport = GameExport::GAME_ICARUS_LERP2POS;
}
