use crate::ffi::GameExport;

use crate::boundary::generic::{InboundVmCall, RawVmCallArgs};

/// `GAME_CLIENT_CONNECT` inbound executable-to-game `vmMain` call.
pub struct GameClientConnect;

impl InboundVmCall for GameClientConnect {
    type Command = GameExport;
    type Args = RawVmCallArgs;
    type Output = isize;

    const COMMAND: GameExport = GameExport::GAME_CLIENT_CONNECT;
}
