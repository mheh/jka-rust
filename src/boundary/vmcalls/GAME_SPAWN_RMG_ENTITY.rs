use crate::ffi::GameExport;

use super::super::generic::{InboundVmCall, RawVmCallArgs};

/// `GAME_SPAWN_RMG_ENTITY` inbound executable-to-game `vmMain` call.
pub struct GameSpawnRmgEntity;

impl InboundVmCall for GameSpawnRmgEntity {
    type Args = RawVmCallArgs;
    type Output = isize;

    const COMMAND: GameExport = GameExport::GAME_SPAWN_RMG_ENTITY;
}
