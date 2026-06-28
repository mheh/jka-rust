use crate::ffi::GameExport;

use crate::boundary::generic::{InboundVmCall, RawVmCallArgs};

/// `GAME_NAV_FINDCOMBATPOINTWAYPOINTS` inbound executable-to-game `vmMain` call.
pub struct GameNavFindcombatpointwaypoints;

impl InboundVmCall for GameNavFindcombatpointwaypoints {
    type Command = GameExport;
    type Args = RawVmCallArgs;
    type Output = isize;

    const COMMAND: GameExport = GameExport::GAME_NAV_FINDCOMBATPOINTWAYPOINTS;
}
