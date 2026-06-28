use crate::ffi::GameExport;

use super::super::generic::{InboundVmCall, RawVmCallArgs};

/// `GAME_ICARUS_GETSTRING` inbound executable-to-game `vmMain` call.
pub struct GameIcarusGetstring;

impl InboundVmCall for GameIcarusGetstring {
    type Args = RawVmCallArgs;
    type Output = isize;

    const COMMAND: GameExport = GameExport::GAME_ICARUS_GETSTRING;
}
