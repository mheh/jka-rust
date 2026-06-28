use crate::ffi::GameExport;

use super::super::generic::{InboundVmCall, RawVmCallArgs};

/// `GAME_ICARUS_GETTAG` inbound executable-to-game `vmMain` call.
pub struct GameIcarusGettag;

impl InboundVmCall for GameIcarusGettag {
    type Args = RawVmCallArgs;
    type Output = isize;

    const COMMAND: GameExport = GameExport::GAME_ICARUS_GETTAG;
}
