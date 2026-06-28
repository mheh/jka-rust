use crate::ffi::GameExport;

use super::super::generic::{InboundVmCall, RawVmCallArgs};

/// `GAME_NAV_FINDCOMBATPOINTWAYPOINTS` inbound executable-to-game `vmMain` call.
pub struct GameNavFindcombatpointwaypoints;

impl InboundVmCall for GameNavFindcombatpointwaypoints {
    type Args = RawVmCallArgs;
    type Output = isize;

    const COMMAND: GameExport = GameExport::GAME_NAV_FINDCOMBATPOINTWAYPOINTS;
}
