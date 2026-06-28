use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_SPAWN_RMG_ENTITY` inbound executable-to-game `vmMain` call.
pub struct GameSpawnRmgEntity;

impl InboundVmCall for GameSpawnRmgEntity {
    type Command = GameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: GameExport = GameExport::GAME_SPAWN_RMG_ENTITY;
}
