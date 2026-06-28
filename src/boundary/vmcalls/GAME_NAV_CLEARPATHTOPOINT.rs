use crate::ffi::GameExport;

use super::super::generic::{InboundVmCall, RawVmCallArgs};

/// `GAME_NAV_CLEARPATHTOPOINT` inbound executable-to-game `vmMain` call.
pub struct GameNavClearpathtopoint;

impl InboundVmCall for GameNavClearpathtopoint {
    type Args = RawVmCallArgs;
    type Output = isize;

    const COMMAND: GameExport = GameExport::GAME_NAV_CLEARPATHTOPOINT;
}
