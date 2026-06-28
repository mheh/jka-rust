use crate::ffi::GameExport;

use crate::boundary::generic::{InboundVmCall, RawVmCallArgs};

/// `GAME_NAV_CHECKNODEFAILEDFORENT` inbound executable-to-game `vmMain` call.
pub struct GameNavChecknodefailedforent;

impl InboundVmCall for GameNavChecknodefailedforent {
    type Command = GameExport;
    type Args = RawVmCallArgs;
    type Output = isize;

    const COMMAND: GameExport = GameExport::GAME_NAV_CHECKNODEFAILEDFORENT;
}
