use crate::ffi::GameExport;

use super::super::generic::{InboundVmCall, RawVmCallArgs};

/// `GAME_ICARUS_GETSETIDFORSTRING` inbound executable-to-game `vmMain` call.
pub struct GameIcarusGetsetidforstring;

impl InboundVmCall for GameIcarusGetsetidforstring {
    type Args = RawVmCallArgs;
    type Output = isize;

    const COMMAND: GameExport = GameExport::GAME_ICARUS_GETSETIDFORSTRING;
}
