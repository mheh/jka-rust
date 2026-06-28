use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_CLIENT_CONNECT` inbound executable-to-game `vmMain` call.
pub struct GameClientConnect;

impl InboundVmCall for GameClientConnect {
    type Command = GameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: GameExport = GameExport::GAME_CLIENT_CONNECT;
}
