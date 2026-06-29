use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_SPAWN_RMG_ENTITY` MP game exports vmMain boundary token.
///
/// Raven: rwwRMG - added
/// Raven: rww - icarus callbacks
/// Source: `oracle/oracle/codemp/game/g_public.h:768`
pub struct GameSpawnRmgEntity;

impl InboundVmCall for GameSpawnRmgEntity {
    type Command = GameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: GameExport = GameExport::GAME_SPAWN_RMG_ENTITY;
}
