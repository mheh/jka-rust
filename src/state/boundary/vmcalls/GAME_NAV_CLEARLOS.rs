use crate::ffi::GameExport;

use super::super::generic::{InboundVmCall, RawVmCallArgs};

/// `GAME_NAV_CLEARLOS` inbound executable-to-game `vmMain` call.
pub struct GameNavClearlos;

impl InboundVmCall for GameNavClearlos {
    type Args = RawVmCallArgs;
    type Output = isize;

    const COMMAND: GameExport = GameExport::GAME_NAV_CLEARLOS;
}
