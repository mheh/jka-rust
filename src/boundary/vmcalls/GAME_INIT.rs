use crate::ffi::GameExport;

use super::super::generic::{InboundVmCall, RawVmCallArgs};

/// `GAME_INIT` inbound executable-to-game `vmMain` call.
pub struct GameInit;

impl InboundVmCall for GameInit {
    type Args = RawVmCallArgs;
    type Output = isize;

    const COMMAND: GameExport = GameExport::GAME_INIT;
}
