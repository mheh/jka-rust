use crate::ffi::GameExport;

use super::super::generic::{InboundVmCall, RawVmCallArgs};

/// `GAME_NAV_CLEARPATHBETWEENPOINTS` inbound executable-to-game `vmMain` call.
pub struct GameNavClearpathbetweenpoints;

impl InboundVmCall for GameNavClearpathbetweenpoints {
    type Args = RawVmCallArgs;
    type Output = isize;

    const COMMAND: GameExport = GameExport::GAME_NAV_CLEARPATHBETWEENPOINTS;
}
