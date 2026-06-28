use crate::ffi::GameExport;

use super::super::generic::{InboundVmCall, RawVmCallArgs};

/// `GAME_NAV_CHECKNODEFAILEDFORENT` inbound executable-to-game `vmMain` call.
pub struct GameNavChecknodefailedforent;

impl InboundVmCall for GameNavChecknodefailedforent {
    type Args = RawVmCallArgs;
    type Output = isize;

    const COMMAND: GameExport = GameExport::GAME_NAV_CHECKNODEFAILEDFORENT;
}
