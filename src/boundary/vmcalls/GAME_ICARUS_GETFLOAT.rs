use crate::ffi::GameExport;

use super::super::generic::{InboundVmCall, RawVmCallArgs};

/// `GAME_ICARUS_GETFLOAT` inbound executable-to-game `vmMain` call.
pub struct GameIcarusGetfloat;

impl InboundVmCall for GameIcarusGetfloat {
    type Args = RawVmCallArgs;
    type Output = isize;

    const COMMAND: GameExport = GameExport::GAME_ICARUS_GETFLOAT;
}
